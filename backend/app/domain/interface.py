"""Base class for all interfaces in the domain layer."""

from abc import ABC


class Interface(ABC):
    """Marker base class for interfaces.

    Every subclass of ``Interface`` is considered a pure interface.
    All its methods must be decorated with ``@abstractmethod``.
    """
