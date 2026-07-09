"""Info endpoint — exposes sidecar metadata and optional dependency status."""

import sys

from fastapi import APIRouter

from app.models import InfoResponse

router = APIRouter()


def _check_langgraph() -> bool:
    try:
        import langgraph  # noqa: F401

        return True
    except ImportError:
        return False


def _check_powermem() -> bool:
    try:
        import powermem  # noqa: F401

        return True
    except ImportError:
        return False


def _check_deepagents() -> bool:
    try:
        import deepagents  # noqa: F401

        return True
    except ImportError:
        return False


@router.get("/info", response_model=InfoResponse)
async def get_info() -> InfoResponse:
    """Return sidecar metadata and optional dependency availability."""
    return InfoResponse(
        name="misaka-agent",
        version="0.1.0",
        python_version=sys.version,
        langgraph_available=_check_langgraph() or _check_deepagents(),
        powermem_available=_check_powermem(),
    )
