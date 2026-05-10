"""Health check endpoint with enhanced status reporting."""

import time

from fastapi import APIRouter, Request

from app.models import HealthResponse

router = APIRouter()

SIDECAR_VERSION = "0.1.0"


@router.get("/health", response_model=HealthResponse)
async def health_check(request: Request) -> HealthResponse:
    """Return health status with version, uptime, and capabilities."""
    startup_time: float = getattr(request.app.state, "startup_time", 0.0)
    uptime = time.time() - startup_time if startup_time > 0 else 0.0

    capabilities: list[str] = ["health", "info"]

    try:
        import langgraph  # noqa: F401
        capabilities.append("agent")
    except ImportError:
        pass

    try:
        import powermem  # noqa: F401
        capabilities.append("memory")
    except ImportError:
        pass

    agent_ready = "agent" in capabilities

    return HealthResponse(
        status="ok",
        service="misaka-agent",
        version=SIDECAR_VERSION,
        uptime_seconds=round(uptime, 2),
        capabilities=capabilities,
        agent_ready=agent_ready,
    )
