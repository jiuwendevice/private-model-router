from typing import Any, Optional

def importorskip(
    name: str,
    minversion: Optional[str] = None,
    *,
    reason: Optional[str] = None,
) -> Any: ...
