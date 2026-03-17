"""Hyperlight guest-side helpers, available to user code via `from hyperlight import call_tool`."""

from sandbox_executor import _call_tool as call_tool

__all__ = ["call_tool"]
