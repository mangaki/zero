from collections import Counter, defaultdict

import numpy as np
from scipy.sparse import coo_matrix, diags
from scipy.sparse.linalg import norm

from zero.recommendation_algorithm import (RecommendationAlgorithm,
                                           register_algorithm)


def normalize(X):
    norms = norm(X, axis=1)
    norms[norms == 0] = 1.
    return diags(1 / norms) @ X


def cosine_similarity(X, Y=None):
    X = normalize(X)
    if Y is None:
        Y = X
    else:
        Y = normalize(Y)
    return (X @ Y.T).toarray()


def mean_of_nonzero(X, cols):
    X_csc = X.tocsc()
    sums = X_csc[:, cols].sum(axis=0).A1
    counts = np.diff(X_csc.indptr)[cols]
    counts[counts == 0] = 1.
    return sums / counts


@register_algorithm('knn')
class MangakiKNN(RecommendationAlgorithm):
    def __init__(self, nb_neighbors=20, rated_by_neighbors_at_least=3,
                 missing_is_mean=True, weighted_neighbors=False):
        super().__init__()
        self.M = None
        self.nb_neighbors = nb_neighbors
        self.rated_by_neighbors_at_least = rated_by_neighbors_at_least
        self.missing_is_mean = missing_is_mean
        self.weighted_neighbors = weighted_neighbors
        self.closest = {}
        self.rated_works = {}
        self.mean_score = {}
        self.ratings = {}
        self.sum_ratings = {}
        self.nb_ratings = {}

    @property
    def is_serializable(self):
        return True

    def get_neighbors(self, user_ids=None):
        neighbors = []
        if user_ids is None:
            score = cosine_similarity(self.M)  # All pairwise similarities
            user_ids = range(self.nb_users)
        else:
            score = cosine_similarity(self.M[user_ids], self.M)
        for i, user_id in enumerate(user_ids):
            if self.nb_neighbors < self.nb_users - 1:
                # Do not select the user itself while looking at neighbors
                score[i][user_id] = float('-inf')
                # Put top NB_NEIGHBORS user indices at the end of array,
                # no matter their order; then, slice them!
                neighbor_ids = (
                    score[i]
                    .argpartition(-self.nb_neighbors - 1)
                    [-self.nb_neighbors - 1:-1]
                )
            else:
                neighbor_ids = list(range(self.nb_users))
                neighbor_ids.remove(user_id)
            neighbors.append(neighbor_ids)
            self.closest[user_id] = {}
            for neighbor_id in neighbor_ids:
                self.closest[user_id][neighbor_id] = score[i, neighbor_id]
        return neighbors

    def fit(self, X, y, whole_dataset=False):
        self.ratings = defaultdict(dict)
        self.sum_ratings = Counter()
        self.nb_ratings = Counter()
        users, works = zip(*list(X))
        # Might take some time, but coo is efficient for creating matrices
        self.M = coo_matrix((y, (users, works)),
                            shape=(self.nb_users, self.nb_works)).astype(
                                np.float64)
        # knn.M should be CSR for faster arithmetic operations
        self.M = self.M.tocsr()
        for (user_id, work_id), rating in zip(X, y):
            self.ratings[user_id][work_id] = rating
            self.nb_ratings[work_id] += 1
            self.sum_ratings[work_id] += rating
        for work_id in self.nb_ratings:
            self.mean_score[work_id] = (self.sum_ratings[work_id] /
                                        self.nb_ratings[work_id])

    def fit_single_user(self, rated_works, ratings):
        nb_rated = len(rated_works)
        nb_neighbors = min(self.nb_neighbors, self.nb_users)
        user_ratings = coo_matrix((ratings, ([0.] * nb_rated, rated_works)),
                                  shape=(1, self.nb_works)).tocsr()
        score = cosine_similarity(user_ratings, self.M).reshape(-1)
        return np.argpartition(score, -nb_neighbors)[-nb_neighbors:]

    def predict(self, X):
        # Compute only relevant neighbors
        self.get_neighbors(list(set(X[:, 0])))
        y = []
        for my_user_id, work_id in X:
            weight = 0
            predicted_rating = 0
            nb_neighbors_that_rated_it = 0
            for user_id in self.closest[my_user_id]:
                their_sim_score = self.closest[my_user_id][user_id]
                if self.missing_is_mean:
                    if work_id in self.ratings[user_id]:
                        their_rating = self.ratings[user_id][work_id]
                        nb_neighbors_that_rated_it += 1
                    else:
                        # In case KNN was not trained on this work
                        their_rating = self.mean_score.get(work_id, 0)
                else:
                    their_rating = self.ratings[user_id].get(work_id)
                    if their_rating is None:
                        continue  # Skip
                if self.weighted_neighbors:
                    predicted_rating += their_sim_score * their_rating
                    weight += their_sim_score
                else:
                    predicted_rating += their_rating
                    weight += 1
            if nb_neighbors_that_rated_it < self.rated_by_neighbors_at_least:
                predicted_rating = 0
            if weight > 0:
                predicted_rating /= weight
            y.append(predicted_rating)
        return np.array(y)

    def predict_single_user(self, work_ids, neighbor_ids):
        return mean_of_nonzero(self.M[neighbor_ids], work_ids)

    def __str__(self):
        return '[KNN] NB_NEIGHBORS = %d' % self.nb_neighbors

    def get_shortname(self):
        return 'knn-%d' % self.nb_neighbors
