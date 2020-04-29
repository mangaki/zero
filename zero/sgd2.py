import numpy as np
from scipy.sparse import coo_matrix, hstack, diags
import time

from zero.recommendation_algorithm import (RecommendationAlgorithm,
                                           register_algorithm)


def onehotize(col, depth):
    nb_events = len(col)
    rows = list(range(nb_events))
    return coo_matrix(([1] * nb_events, (rows, col)), shape=(nb_events, depth))


@register_algorithm('sgd2')
class MangakiSGD2(RecommendationAlgorithm):
    def __init__(self, nb_components=20, nb_iterations=10,
                 gamma=0.01, lambda_=0.1, batches=400):
        super().__init__()
        self.nb_components = nb_components
        self.nb_iterations = nb_iterations
        self.gamma = gamma
        self.lambda_ = lambda_
        self.batches = batches

    def fit(self, X, y):
        N = self.nb_users + self.nb_works
        self.w = np.random.random(N)
        self.V = np.random.random((N, self.nb_components))
        X_users = onehotize(X[:, 0], self.nb_users)
        X_works = onehotize(X[:, 1], self.nb_works)
        X_fm = hstack([X_users, X_works]).tocsr()
        batch_size = max(1, len(X) // self.batches)
        for epoch in range(self.nb_iterations):
            step = 0
            dt = time.time()
            batch = np.random.permutation(len(X))
            for i in range(self.batches):
                X_batch = X_fm[batch[i * batch_size:(i + 1) * batch_size]]
                X_bT = X_batch.T.tocsr()
                y_batch = y[batch[i * batch_size:(i + 1) * batch_size]]
                pred_batch = self.predict_fm(X_batch)
                error_batch = pred_batch - y_batch
                error_feat = X_bT.dot(error_batch)

                w_grad = error_feat / batch_size + self.lambda_ * self.w
                V_grad = ((1 / batch_size + self.lambda_) *
                          (X_bT @ diags(error_batch) @ X_bT.T -
                           diags(error_feat))) @ self.V

                self.w -= self.gamma * w_grad
                self.V -= self.gamma * V_grad
                step += 1
            print('elapsed', time.time() - dt)
            self.compute_metrics()

    def fit_single_user(self, rated_works, ratings):
        pass

    def predict(self, X):
        X_users = onehotize(X[:, 0], self.nb_users)
        X_works = onehotize(X[:, 1], self.nb_works)
        X_fm = hstack([X_users, X_works]).tocsr()
        return self.predict_fm(X_fm)

    def predict_fm(self, X):
        return X @ self.w + 1/2 * (np.sum((X @ self.V) ** 2 -
                                          X @ (self.V ** 2), axis=1))

    def predict_single_user(self, work_ids, user_parameters):
        pass

    @property
    def is_serializable(self):
        return False  # Not yet, but easy to do

    def __str__(self):
        return '[SGD2] NB_COMPONENTS = %d' % self.nb_components

    def get_shortname(self):
        return 'sgd2-%d' % self.nb_components
