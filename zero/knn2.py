import numpy as np
from scipy.sparse import coo_matrix
from sklearn.metrics.pairwise import cosine_similarity

from zero.recommendation_algorithm import (RecommendationAlgorithm,
                                           register_algorithm)


@register_algorithm('knn2')
class MangakiKNN2(RecommendationAlgorithm):
    '''
    Toy implementation (not usable in production) of KNN for the mere
    sake of science.
    :math:`N` users, :math:`M` ~ 10k works, :math:`P` ~ 300k user-work pairs, :math:`K` neighbors.

    Algorithm:
    For each user-work pair (over all P pairs):
    - Find closest raters of user *who rated this work* (takes :math:`O(M \log M)`)
    - Compute their average rating (takes :math:`O(K)`)
    Complexity: :math:`O(P (M \log M + K))` => Oops!
    '''
    def __init__(self, nb_neighbors=20, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.nb_neighbors = nb_neighbors
        self.ratings = None

    @property
    def is_serializable(self):
        return True

    def fit(self, X, y, whole_dataset=False):
        user_ids = X[:, 0]
        work_ids = X[:, 1]
        self.ratings = coo_matrix((y, (user_ids, work_ids)),
                                  shape=(self.nb_users, self.nb_works))
        self.ratings_by_user = self.ratings.tocsr()
        self.ratings_by_work = self.ratings.tocsc()
        self.user_similarity = cosine_similarity(self.ratings_by_user)

    def predict(self, X):
        y = []
        for user_id, work_id in X:
            closest_raters = list(self.ratings_by_work[:, work_id].indices)
            closest_raters.sort(
                key=lambda rater_id: self.user_similarity[user_id, rater_id],
                reverse=True)
            neighbor_ids = closest_raters[:self.nb_neighbors]
            rating = self.ratings_by_work[neighbor_ids, work_id].mean()
            y.append(rating)
        return np.array(y)

    def __str__(self):
        return '[KNN2] NB_NEIGHBORS = %d' % self.nb_neighbors

    def get_shortname(self):
        return 'knn2-%d' % self.nb_neighbors
