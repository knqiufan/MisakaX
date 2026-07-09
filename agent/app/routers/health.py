"""Health check endpoint with enhanced status reporting."""

import time

from fastapi import APIRouter, Request

from app.models import HealthResponse

router = APIRouter()

SIDECAR_VERSION = "0.1.0"


def _agent_module_ready() -> bool:
    try:
        from app.agent import build_agent  # noqa: F401

        return True
    except Exception:
        return False


def _deepagents_available() -> bool:
    try:
        import deepagents  # noqa: F401

        return True
    except ImportError:
        return False


def _langgraph_available() -> bool:
    try:
        import langgraph  # noqa: F401

        return True
    except ImportError:
        return False


def _powermem_available() -> bool:
    try:
        import powermem  # noqa: F401

        return True
    except ImportError:
        return False


@router.get("/health", response_model=HealthResponse)
async def health_check(request: Request) -> HealthResponse:
    """Return health status with version, uptime, and capabilities."""
    startup_time: float = getattr(request.app.state, "startup_time", 0.0)
    uptime = time.time() - startup_time if startup_time > 0 else 0.0

    capabilities: list[str] = ["health", "info"]
    if _langgraph_available() or _deepagents_available() or _agent_module_ready():
        capabilities.append("agent")
    if _powermem_available():
        capabilities.append("memory")

    agent_ready = _agent_module_ready() and (
        _deepagents_available() or _langgraph_available()
    )

    return HealthResponse(
        status="ok",
        service="misaka-agent",
        version=SIDECAR_VERSION,
        uptime_seconds=round(uptime, 2),
        capabilities=capabilities,
        agent_ready=agent_ready,
    )
