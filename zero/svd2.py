"""
Mangaki sparse SVD.
Author: Jill-Jênn Vie, 2020
"""
import numpy as np
from scipy.sparse import csr_matrix, diags
from scipy.sparse.linalg import svds
from zero.recommendation_algorithm import (RecommendationAlgorithm,
                                           register_algorithm)


def remove_mean(sp_matrix):
    '''
    For each row of a sparse matrix,
    remove the mean of nonzero elements
    from the nonzero elements.
    '''
    mask = sp_matrix.copy()
    mask.data = np.ones_like(sp_matrix.data)
    row_sums = sp_matrix.sum(axis=1).A1
    row_counts = mask.sum(axis=1).A1
    row_counts[row_counts == 0] = 1
    means = row_sums / row_counts
    return sp_matrix - diags(means) * mask, means


@register_algorithm('svd2', {'nb_components': 20})
class MangakiSVD2(RecommendationAlgorithm):
    '''
    Implementation of SVD with sparse matrices.
    It does not compute the whole matrix for recommendations
    but the production should be able to do sparse matrix
    operations effectively.
    It is 7x faster than svd.py, and it only relies on numpy/scipy.
    '''
    def __init__(self, nb_components=20):
        super().__init__()
        self.U = None
        self.sigma = None
        self.VT = None
        self.nb_components = nb_components
        self.means = None

    @property
    def is_serializable(self):
        """
        Check whether we can save the model.
        """
        return True

    def make_matrix(self, X, y):
        """
        Make a sparse matrix out of X and y.
        X is a matrix of pairs (user_id, item_id),
        y are real values of ratings.
        """
        rows = X[:, 0]
        cols = X[:, 1]
        ratings = csr_matrix((y, (rows, cols)),
                             shape=(self.nb_users, self.nb_works))
        ratings, means = remove_mean(ratings)
        return ratings, means

    def fit(self, X, y):
        """
        Fit the SVD.
        """
        if self.verbose_level:
            print("Computing M: (%i × %i)" % (self.nb_users, self.nb_works))
        matrix, self.means = self.make_matrix(X, y)

        self.chrono.save('fill and center matrix')

        self.U, self.sigma, self.VT = svds(matrix, k=self.nb_components)

        if self.verbose_level:
            print('Shapes', self.U.shape, self.sigma.shape, self.VT.shape)

        self.chrono.save('factor matrix')

    def fit_single_user(self, rated_works, ratings):
        """
        Fit the SVD for a single user.
        """
        nb_components = min(self.nb_components, self.sigma.shape[0])
        mean_user = np.mean(ratings)
        ratings -= mean_user
        Ru = np.array(ratings, ndmin=2).T
        Vu = np.diag(self.sigma).dot(self.VT[:, rated_works])
        Gu = 0.1 * len(rated_works) * np.eye(nb_components)
        feat_user = np.linalg.solve(Vu.dot(Vu.T) + Gu, Vu.dot(Ru)).reshape(-1)
        return mean_user, feat_user

    def predict(self, X):
        """
        Predict ratings for user, item pairs.
        """
        Us = self.U * self.sigma
        return ((Us[X[:, 0]] * self.VT.T[X[:, 1]]).sum(axis=1) +
                self.means[X[:, 0]])

    def predict_single_user(self, work_ids, user_parameters):
        """
        Predict ratings for a single user.
        """
        mean, U = user_parameters
        return mean + U.dot(self.VT[:, work_ids])

    def get_shortname(self):
        """
        Short name useful for logging output.
        """
        return 'svd2-%d' % self.nb_components
