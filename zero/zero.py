from zero.recommendation_algorithm import (RecommendationAlgorithm,
                                           register_algorithm)
import numpy as np


@register_algorithm('zero')
class MangakiZero(RecommendationAlgorithm):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

    def fit(self, X, y):
        pass

    def predict(self, X):
        return np.zeros(len(X))

    def get_shortname(self):
        return 'zero'
