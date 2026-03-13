from app.models.job import CompressionAlgorithm
from app.services.compression_strategies.base import CompressionStrategy
from app.services.compression_strategies.sevenz import SevenZipCompressionStrategy
from app.services.compression_strategies.zip import ZipCompressionStrategy


def build_strategy_registry() -> dict[CompressionAlgorithm, CompressionStrategy]:
    strategies = [SevenZipCompressionStrategy(), ZipCompressionStrategy()]
    return {strategy.algorithm: strategy for strategy in strategies}
