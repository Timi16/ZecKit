"""
ZecKit Faucet - Statistics Endpoint
Provides faucet usage statistics with REAL uptime
"""
from flask import Blueprint, jsonify, current_app, request
from datetime import datetime
import logging

logger = logging.getLogger(__name__)

stats_bp = Blueprint('stats', __name__)


def _format_uptime(seconds: float) -> str:
    """Convert seconds into clean format: 3d 12h 45m 8s"""
    if seconds < 0:
        return "0s"

    days = int(seconds // 86400)
    hours = int((seconds % 86400) // 3600)
    minutes = int((seconds % 3600) // 60)
    secs = int(seconds % 60)

    parts = []
    if days:   parts.append(f"{days}d")
    if hours:  parts.append(f"{hours}h")
    if minutes:parts.append(f"{minutes}m")
    parts.append(f"{secs}s")
    return " ".join(parts)


@stats_bp.route('/stats', methods=['GET'])
def get_stats():
    """
    Get faucet statistics (now with real uptime!)
    """
    wallet = current_app.faucet_wallet
    
    if not wallet or not wallet.is_loaded():
        return jsonify({
            "error": "Faucet wallet not available",
            "code": "FAUCET_UNAVAILABLE"
        }), 503
    
    # Wallet stats
    wallet_stats = wallet.get_stats()
    tx_history = wallet.get_transaction_history(limit=1000)
    last_request = tx_history[-1].get('timestamp') if tx_history else None

    # REAL UPTIME — this works because we set app.start_time in main.py
    uptime_seconds = (datetime.utcnow() - current_app.start_time).total_seconds()

    stats = {
        "faucet_address": wallet_stats['address'],
        "current_balance": wallet_stats['current_balance'],
        "total_requests": wallet_stats['total_transactions'],
        "total_sent": wallet_stats['total_sent'],
        "created_at": wallet_stats['created_at'],
        "last_request": last_request,
        "uptime": _format_uptime(uptime_seconds),        # e.g. "2d 9h 34m 12s"
        "uptime_seconds": int(uptime_seconds),           # for bots/monitoring
        "version": "0.1.0"
    }
    
    return jsonify(stats), 200


@stats_bp.route('/history', methods=['GET'])
def get_history():
    """
    Get recent transaction history
    """
    wallet = current_app.faucet_wallet
    
    if not wallet or not wallet.is_loaded():
        return jsonify({
            "error": "Faucet wallet not available",
            "code": "FAUCET_UNAVAILABLE"
        }), 503
    
    try:
        limit = int(request.args.get('limit', 100))
        limit = min(max(1, limit), 1000)
    except ValueError:
        limit = 100
    
    history = wallet.get_transaction_history(limit=limit)
    
    return jsonify({
        "count": len(history),
        "limit": limit,
        "transactions": history
    }), 200