"""
Factory for the compression strategy registry.

Instantiates all available strategies and exposes them in a dictionary
keyed by ``CompressionAlgorithm`` for use by ``CompressionService``.
"""

from app.models.job import CompressionAlgorithm
from app.services.compression_strategies.base import CompressionStrategy
from app.services.compression_strategies.sevenz import SevenZipCompressionStrategy
from app.services.compression_strategies.zip import ZipCompressionStrategy


def build_strategy_registry() -> dict[CompressionAlgorithm, CompressionStrategy]:
    """
    Build and return the algorithm → strategy mapping.

    Each strategy self-registers via its ``algorithm`` property.
    To add a new algorithm, simply include its strategy in the list.
    """
    strategies = [SevenZipCompressionStrategy(), ZipCompressionStrategy()]
    return {strategy.algorithm: strategy for strategy in strategies}
