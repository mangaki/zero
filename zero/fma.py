from scipy.sparse import coo_matrix, hstack
import numpy as np
import pywFM
import os

from zero.recommendation_algorithm import (RecommendationAlgorithm,
                                           register_algorithm)


@register_algorithm('fma', {'rank': 20, 'nb_iterations': 20})
class MangakiFMA(RecommendationAlgorithm):
    def __init__(self, rank=20, nb_iterations=20):
        super().__init__()
        self.rank = rank
        self.nb_iterations = nb_iterations
        self.fm = None
        self.T = None

    @property
    def is_serializable(self):
        return True

    def prepare_fm(self, X):
        if self.T is None:
            self.load_tags()

        nb_samples = len(X)
        user_ids = X[:, 0]
        work_ids = X[:, 1].astype(np.int32)  # Otherwise we can't slice self.T
        # For the k-th user_id-work_id pair, we need (k, user_id)
        # and (k, N + work_id), so two copies of range(nb_samples)
        rows = list(range(nb_samples)) * 2
        cols = np.concatenate((user_ids, self.nb_users + work_ids))
        X_fm = coo_matrix(([1] * (2 * nb_samples), (rows, cols)),
                          shape=(nb_samples, self.nb_users + self.nb_works)
                          ).tocsr()
        if self.T.nnz == 0:
            return X_fm
        X_tags = self.T[work_ids].round()
        X_full = hstack((X_fm, X_tags))
        return X_full

    def fit(self, X, y):
        # Should not be done in production :)
        # Otherwise you should also install libFM:
        # https://github.com/srendle/libfm
        X_fm = self.prepare_fm(X)
        self.chrono.save('prepare data in sparse FM format')

        os.environ['LIBFM_PATH'] = 'XXX'  # If applicable
        fm = pywFM.FM(task='regression', num_iter=self.nb_iterations,
                      k2=self.rank, rlog=True)  # MCMC method
        # rlog contains the RMSE at each epoch, we do not need it here
        model = fm.run(X_fm, y, self.prepare_fm(self.X_test), self.y_test)
        self.chrono.save('train FM')

        nb_agents = self.nb_users + self.nb_works + self.nb_tags
        current = len(model.weights)

        if model.global_bias is None:
            # Train failed (for example, libfm does not exist)
            self.mu = 0
            self.W = np.random.random(nb_agents)
            self.V = np.random.random((nb_agents, self.rank))
        else:
            self.mu = model.global_bias
            # Just in case X_fm had too many zero columns on the right
            self.W = np.pad(np.array(model.weights), (0, nb_agents - current),
                            mode='constant')
            self.V = np.pad(model.pairwise_interactions,
                            [(0, nb_agents - current), (0, 0)],
                            mode='constant')
        self.V2 = np.power(self.V, 2)
        self.metrics['test']['rmse'] = list(model.rlog['rmse'])

    def predict(self, X):
        X_fm = self.prepare_fm(X)
        X2_fm = X_fm.copy()
        X2_fm.data **= 2

        y_pred = (self.mu + X_fm.dot(self.W) +
                  0.5 * (np.power(X_fm.dot(self.V), 2).sum(axis=1)
                         - X2_fm.dot(self.V2).sum(axis=1)))
        return y_pred

    def get_shortname(self):
        return 'fma-%d' % self.rank
