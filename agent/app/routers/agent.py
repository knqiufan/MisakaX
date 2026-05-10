"""Agent chat endpoints — placeholder stubs for Phase 4 implementation."""

from fastapi import APIRouter, HTTPException

from app.models import ChatRequest

router = APIRouter(prefix="/agent", tags=["agent"])


@router.post("/chat")
async def agent_chat(request: ChatRequest):
    """Placeholder: full agent chat will be implemented in Phase 4."""
    raise HTTPException(
        status_code=501,
        detail="Agent chat is not yet implemented. Coming in Phase 4.",
    )


@router.post("/stream")
async def agent_stream(request: ChatRequest):
    """Placeholder: streaming agent chat will be implemented in Phase 4."""
    raise HTTPException(
        status_code=501,
        detail="Agent streaming is not yet implemented. Coming in Phase 4.",
    )
