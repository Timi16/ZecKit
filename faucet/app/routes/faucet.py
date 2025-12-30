"""
ZecKit Faucet - Funding Request Endpoint
"""
from flask import Blueprint, jsonify, request, current_app
from datetime import datetime
import requests

faucet_bp = Blueprint('faucet', __name__)


def validate_address_via_zebra_rpc(address: str) -> tuple:
    """
    Validate Zcash address using Zebra's validateaddress RPC.
    This ensures we're validating against actual Zcash protocol rules,
    not just regex patterns.
    
    Args:
        address: Zcash address to validate
        
    Returns:
        tuple: (is_valid: bool, message: str)
    """
    if not address or not isinstance(address, str):
        return False, "Address is required"
    
    try:
        # Use Zebra RPC to validate (as Pacu requested)
        zebra_rpc_url = "http://zebra:18232"
        
        response = requests.post(
            zebra_rpc_url,
            json={
                "jsonrpc": "2.0",
                "id": "validate_addr",
                "method": "validateaddress",
                "params": [address]
            },
            auth=("zcashrpc", "notsecure"),
            timeout=5
        )
        
        if response.status_code != 200:
            current_app.logger.error(f"Zebra RPC error: HTTP {response.status_code}")
            return False, "Unable to validate address - RPC unavailable"
        
        rpc_result = response.json()
        
        # Check for RPC errors
        if 'error' in rpc_result and rpc_result['error'] is not None:
            error_msg = rpc_result['error'].get('message', 'Unknown RPC error')
            current_app.logger.error(f"Zebra RPC error: {error_msg}")
            return False, f"Address validation failed: {error_msg}"
        
        result = rpc_result.get('result', {})
        
        # Zebra returns isvalid field
        if not result.get('isvalid', False):
            return False, "Invalid Zcash address"
        
        # Additional check: ensure it's a regtest address
        # Regtest addresses have specific prefixes
        valid_prefixes = ('tm', 'uregtest', 'zregtestsapling')
        
        if not any(address.startswith(prefix) for prefix in valid_prefixes):
            current_app.logger.warning(
                f"Address {address[:12]}... does not have regtest prefix"
            )
            return False, "Address is not a valid regtest address"
        
        validated_address = result.get('address', address)
        current_app.logger.info(f"Address validated: {validated_address[:12]}...")
        
        return True, validated_address
        
    except requests.exceptions.Timeout:
        current_app.logger.error("Zebra RPC timeout")
        return False, "Address validation timeout - node not responding"
        
    except requests.exceptions.ConnectionError:
        current_app.logger.error("Cannot connect to Zebra RPC")
        return False, "Cannot connect to validation service"
        
    except Exception as e:
        current_app.logger.error(f"Unexpected validation error: {e}")
        return False, f"Validation error: {str(e)}"


@faucet_bp.route('/request', methods=['POST'])
def request_funds():
    """
    Request test funds from faucet on regtest network
    """
    data = request.get_json()
    if not data:
        return jsonify({"error": "Invalid JSON"}), 400
    
    # Validate address using Zebra RPC (not regex!)
    to_address = data.get('address')
    is_valid, result_or_error = validate_address_via_zebra_rpc(to_address)
    
    if not is_valid:
        return jsonify({"error": result_or_error}), 400
    
    # Use the validated/normalized address
    validated_address = result_or_error
    
    # Get amount
    try:
        amount = float(data.get('amount', current_app.config['FAUCET_AMOUNT_DEFAULT']))
        
        min_amount = current_app.config['FAUCET_AMOUNT_MIN']
        max_amount = current_app.config['FAUCET_AMOUNT_MAX']
        
        if amount < min_amount or amount > max_amount:
            return jsonify({
                "error": f"Amount must be between {min_amount} and {max_amount} ZEC"
            }), 400
    
    except (ValueError, TypeError):
        return jsonify({"error": "Invalid amount"}), 400
    
    # Check wallet ready
    wallet = current_app.faucet_wallet
    if not wallet:
        return jsonify({"error": "Faucet wallet not available"}), 503
    
    # Check balance
    balance = wallet.get_balance()
    if balance < amount:
        return jsonify({
            "error": f"Insufficient faucet balance (available: {balance} ZEC)"
        }), 503
    
    # Send transaction
    try:
        result = wallet.send_to_address(
            to_address=validated_address,
            amount=amount,
            memo=data.get('memo')
        )
        
        if not result.get("success"):
            return jsonify({
                "error": f"Transaction failed: {result.get('error')}"
            }), 500
        
        new_balance = wallet.get_balance()
        
        return jsonify({
            "success": True,
            "txid": result["txid"],
            "address": validated_address,
            "amount": amount,
            "new_balance": float(new_balance),
            "timestamp": result["timestamp"],
            "network": "regtest",
            "message": f"Sent {amount} ZEC on regtest. TXID: {result['txid']}"
        }), 200
    
    except Exception as e:
        current_app.logger.error(f"Transaction error: {e}")
        return jsonify({"error": str(e)}), 500


@faucet_bp.route('/address', methods=['GET'])
def get_faucet_address():
    """Get the faucet's receiving address"""
    wallet = current_app.faucet_wallet
    
    if not wallet:
        return jsonify({"error": "Faucet wallet not available"}), 503
    
    return jsonify({
        "address": wallet.get_address("unified"),
        "balance": float(wallet.get_balance()),
        "network": "regtest"
    }), 200


@faucet_bp.route('/sync', methods=['POST'])
def sync_wallet():
    """Manually trigger wallet sync"""
    wallet = current_app.faucet_wallet
    
    if not wallet:
        return jsonify({"error": "Faucet wallet not available"}), 503
    
    try:
        wallet.sync_wallet()
        return jsonify({
            "success": True,
            "message": "Wallet synced successfully",
            "current_balance": float(wallet.get_balance())
        }), 200
    except Exception as e:
        return jsonify({"error": str(e)}), 500