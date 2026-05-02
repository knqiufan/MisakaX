"""Health check endpoint."""

from fastapi import APIRouter

router = APIRouter()


@router.get("/health")
async def health_check():
    """Return health status."""
    return {"status": "ok", "service": "misaka-agent"}
