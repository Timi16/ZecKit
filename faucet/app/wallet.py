import subprocess
import json
import os
import time
import re
from datetime import datetime
from pathlib import Path

class ZingoWallet:
    def __init__(self, data_dir=None, lightwalletd_uri=None):
        self.data_dir = data_dir or os.getenv('WALLET_DATA_DIR', '/var/zingo')
        self.lightwalletd_uri = lightwalletd_uri or os.getenv('LIGHTWALLETD_URI', 'http://lightwalletd:9067')
        self.history_file = Path(self.data_dir) / "faucet-history.json"
        
        print(f"🔧 ZingoWallet initialized:")
        print(f"  Data dir: {self.data_dir}")
        print(f"  Backend URI: {self.lightwalletd_uri}")
        
    def _run_zingo_cmd(self, command, timeout=30):
        """Run zingo-cli command via docker exec"""
        try:
            wallet_container = os.getenv('WALLET_CONTAINER', 'zeckit-zingo-wallet')
            
            # Use bash -c with echo -e to properly send commands
            cmd_str = f'echo -e "{command}\\nquit" | zingo-cli --data-dir {self.data_dir} --server {self.lightwalletd_uri} --chain regtest --nosync'
            
            cmd = [
                "docker", "exec", wallet_container,
                "bash", "-c", cmd_str
            ]
            
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=timeout
            )
            
            if result.returncode != 0:
                raise Exception(f"Command failed: {result.stderr}")
            
            output = result.stdout.strip()
            
            # Try to parse JSON lines
            for line in output.split('\n'):
                line = line.strip()
                if line.startswith('{') or line.startswith('['):
                    try:
                        return json.loads(line)
                    except:
                        continue
            
            # Return raw output for parsing
            return {"output": output}
            
        except subprocess.TimeoutExpired:
            raise Exception("Command timed out")
        except Exception as e:
            raise Exception(f"Failed to run command: {str(e)}")
    
    def get_balance(self):
        """Get wallet balance in ZEC"""
        try:
            result = self._run_zingo_cmd("balance")
            
            total_zatoshis = 0
            
            # Handle the custom format output
            if isinstance(result, dict) and 'output' in result:
                output = result['output']
                
                # Parse confirmed balances
                patterns = [
                    r'confirmed_transparent_balance:\s*([\d_]+)',
                    r'confirmed_sapling_balance:\s*([\d_]+)',
                    r'confirmed_orchard_balance:\s*([\d_]+)'
                ]
                
                for pattern in patterns:
                    match = re.search(pattern, output)
                    if match:
                        balance_str = match.group(1).replace('_', '')
                        total_zatoshis += int(balance_str)
            
            return total_zatoshis / 100_000_000
            
        except Exception as e:
            print(f"❌ Error getting balance: {e}")
            return 0.0
    
    def get_address(self, address_type="unified"):
        """Get wallet address"""
        try:
            result = self._run_zingo_cmd("addresses")
            
            if isinstance(result, dict) and 'output' in result:
                output = result['output']
                
                # Parse first UA
                match = re.search(r'uregtest1[a-z0-9]{70,}', output)
                if match:
                    return match.group(0)
            
            return None
            
        except Exception as e:
            print(f"❌ Error getting address: {e}")
            return None
    
    def shield_funds(self):
        """Shield all transparent funds to shielded pools"""
        try:
            print("🔄 Shielding transparent funds...")
            result = self._run_zingo_cmd("shield", timeout=60)
            
            if isinstance(result, dict) and 'output' in result:
                output = result['output']
                
                if "error" in output:
                    raise Exception(output)
                
                # Extract TXID from shield response
                match = re.search(r'[0-9a-f]{64}', output)
                if match:
                    txid = match.group(0)
                    print(f"✅ Shielding successful: TXID {txid}")
                    return txid
            
            raise Exception("No TXID in shield response")
            
        except Exception as e:
            print(f"❌ Shielding failed: {e}")
            return None
    
    def send_to_address(self, to_address: str, amount: float, memo: str = None):
        """Send REAL transaction"""
        try:
            # First, shield any transparent funds
            self.shield_funds()
            
            # Wait for shielding to confirm (simple sleep - improve in production)
            time.sleep(30)
            
            # Now send from shielded
            amount_sats = int(amount * 100_000_000)
            cmd = f"send {to_address} {amount_sats}"
            if memo:
                cmd += f' "{memo}"'
            
            result = self._run_zingo_cmd(cmd, timeout=60)
            
            if isinstance(result, dict) and 'output' in result:
                output = result['output']
                
                # Extract TXID
                match = re.search(r'[0-9a-f]{64}', output)
                if match:
                    txid = match.group(0)
                    timestamp = datetime.utcnow().isoformat() + "Z"
                    self._record_transaction(to_address, amount, txid, memo)
                    print(f"✅ Transaction successful: {txid}")
                    return {
                        "success": True,
                        "txid": txid,
                        "timestamp": timestamp
                    }
                
            raise Exception(f"No TXID in response: {result.get('output', '')}")
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    def _record_transaction(self, to_address, amount, txid, memo=""):
        """Record transaction to history"""
        try:
            history = []
            if self.history_file.exists():
                history = json.loads(self.history_file.read_text())
            
            history.append({
                "timestamp": datetime.utcnow().isoformat() + "Z",
                "to_address": to_address,
                "amount": amount,
                "txid": txid,
                "memo": memo
            })
            
            self.history_file.write_text(json.dumps(history, indent=2))
        except Exception as e:
            print(f"⚠️ Failed to record transaction: {e}")
    
    def get_transaction_history(self, limit=100):
        """Get transaction history"""
        try:
            if not self.history_file.exists():
                return []
            
            history = json.loads(self.history_file.read_text())
            return history[-limit:]
        except Exception as e:
            print(f"❌ Error reading history: {e}")
            return []
    
    def get_stats(self):
        """Get wallet statistics"""
        try:
            balance = self.get_balance()
            address = self.get_address()
            history = self.get_transaction_history(limit=10)
            
            return {
                "balance": balance,
                "address": address,
                "transactions_count": len(history),
                "recent_transactions": history[-5:] if history else []
            }
        except Exception as e:
            print(f"❌ Error getting stats: {e}")
            return {
                "balance": 0.0,
                "address": None,
                "transactions_count": 0,
                "recent_transactions": []
            }

# Singleton
_wallet = None

def get_wallet():
    global _wallet
    if _wallet is None:
        _wallet = ZingoWallet()
    return _wallet