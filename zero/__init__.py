from .recommendation_algorithm import (RecommendationAlgorithm,
                                       register_algorithm)
from .dataset import Dataset

from .als import MangakiALS
from .als2 import MangakiALS2
from .knn import MangakiKNN
from .sgd import MangakiSGD
from .sgd2 import MangakiSGD2
from .svd import MangakiSVD
from .zero import MangakiZero
