from .recommendation_algorithm import (RecommendationAlgorithm,
                                       register_algorithm)
from .dataset import Dataset
from .fit_algo import get_algo_backup, get_dataset_backup, fit_algo

from .als import MangakiALS
from .als2 import MangakiALS2
from .als3 import MangakiALS3
from .balse import MangakiBALSE
from .cfm import MangakiCFM
from .efa import MangakiEFA
from .fma import MangakiFMA
from .gbr import MangakiGBR
from .knn import MangakiKNN
from .knn2 import MangakiKNN2
from .lasso import MangakiLASSO
from .pca import MangakiPCA
from .sgd import MangakiSGD
from .ssvd import MangakiSSVD
from .svd import MangakiSVD
from .wals import MangakiWALS
from .xals import MangakiXALS
from .zero import MangakiZero
