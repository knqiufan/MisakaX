"""Health check endpoint with cached startup capability reporting."""

import time
from dataclasses import dataclass

from fastapi import APIRouter, FastAPI, Request

from app.models import HealthResponse

router = APIRouter()

SIDECAR_VERSION = "0.1.0"
_HEALTH_CAPABILITIES_STATE_KEY = "health_capabilities"


@dataclass(frozen=True)
class HealthCapabilities:
    """Static Sidecar capabilities determined once during application startup."""

    agent_ready: bool
    capabilities: tuple[str, ...]


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


def detect_health_capabilities() -> HealthCapabilities:
    """Inspect optional integrations once; installed packages do not change at runtime."""
    agent_ready = _agent_module_ready() and (_deepagents_available() or _langgraph_available())
    capabilities: list[str] = ["health", "info"]
    if agent_ready:
        capabilities.extend(["agent", "agent_stream"])
    if _powermem_available():
        capabilities.append("memory")
    return HealthCapabilities(agent_ready=agent_ready, capabilities=tuple(capabilities))


def cache_health_capabilities(application: FastAPI) -> HealthCapabilities:
    """Populate the application-level health capability snapshot."""
    capabilities = detect_health_capabilities()
    setattr(application.state, _HEALTH_CAPABILITIES_STATE_KEY, capabilities)
    return capabilities


def get_cached_health_capabilities(application: FastAPI) -> HealthCapabilities:
    """Read the startup snapshot, with a safe fallback for direct ASGI tests."""
    cached = getattr(application.state, _HEALTH_CAPABILITIES_STATE_KEY, None)
    if isinstance(cached, HealthCapabilities):
        return cached
    return cache_health_capabilities(application)


@router.get("/health", response_model=HealthResponse)
async def health_check(request: Request) -> HealthResponse:
    """Return health status with version, uptime, and capabilities."""
    startup_time: float = getattr(request.app.state, "startup_time", 0.0)
    uptime = time.time() - startup_time if startup_time > 0 else 0.0

    health_capabilities = get_cached_health_capabilities(request.app)

    return HealthResponse(
        status="ok",
        service="misaka-agent",
        version=SIDECAR_VERSION,
        uptime_seconds=round(uptime, 2),
        capabilities=list(health_capabilities.capabilities),
        agent_ready=health_capabilities.agent_ready,
    )
