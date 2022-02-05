"""
Mangaki sparse SVD.
Author: Jill-Jênn Vie, 2020
"""
import numpy as np
from scipy.sparse import csr_matrix, diags
from scipy.sparse.linalg import svds
from sklearn.neighbors import NearestNeighbors
from zero.recommendation_algorithm import (RecommendationAlgorithm,
                                           register_algorithm)


def remove_mean(sp_matrix, axis=1):
    '''
    For each row (resp. column if axis is 0) of a sparse matrix,
    remove the mean of nonzero elements (resp. the mean of the column)
    from the nonzero elements.
    '''
    mask = sp_matrix.copy()
    mask.data = np.ones_like(sp_matrix.data)
    line_sums = sp_matrix.sum(axis=axis).A1
    if axis == 1:
        line_counts = mask.sum(axis=axis).A1
        line_counts[line_counts == 0] = 1
    else:
        nb_users, nb_works = sp_matrix.shape
        line_counts = nb_users * np.ones(nb_works)
    means = line_sums / line_counts
    if axis == 0:  # Remove from columns
        shifted = sp_matrix - mask * diags(means)
    else:  # Remove from rows
        shifted = sp_matrix - diags(means) * mask
    return shifted, means


@register_algorithm('svdknn')
class MangakiSVDKNN(RecommendationAlgorithm):
    '''
    Implementation of SVD with sparse matrices.
    It does not compute the whole matrix for recommendations
    but the production must be able to do sparse matrix
    operations effectively.
    It is 7x faster than svd1.py, and it only relies on numpy/scipy.
    '''
    def __init__(self, nb_components=20, nb_neighbors=20, nb_iterations=None, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.U = None
        self.sigma = None
        self.VT = None
        self.nb_components = nb_components
        self.nb_neighbors = nb_neighbors
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
        ratings, row_means = remove_mean(ratings, axis=1)
        ratings, col_means = remove_mean(ratings, axis=0)
        return ratings, col_means, row_means

    def fit(self, X, y):
        """
        Fit the SVD.
        """
        if self.verbose_level:
            print("Computing M: (%i × %i)" % (self.nb_users, self.nb_works))
        matrix, self.col_means, self.row_means = self.make_matrix(X, y)

        self.chrono.save('fill and center matrix')

        self.U, self.sigma, self.VT = svds(matrix, k=self.nb_components)

        if self.verbose_level:
            print('Shapes', self.U.shape, self.sigma.shape, self.VT.shape)

        self.chrono.save('factor matrix')

    def fit_single_user(self, rated_works, ratings):
        """
        Fit the SVD for a single user.
        """
        mean_user = np.mean(ratings)
        ratings -= mean_user + self.col_means[rated_works]
        feat_user = ratings @ self.VT.T[rated_works]
        return mean_user, feat_user

    def predict(self, X):
        """
        Predict ratings for user, item pairs.
        """
        knn = NearestNeighbors(n_neighbors=self.nb_neighbors)
        Us = self.U * self.sigma
        knn.fit(Us)
        pred_user_ids = list(set(X[:, 0]))
        neighbor_ids = knn.kneighbors(Us[pred_user_ids], return_distance=False)
        averaged_embedding = np.zeros_like(Us)
        averaged_embedding[pred_user_ids] = Us[neighbor_ids].mean(axis=1)
        return ((averaged_embedding[X[:, 0]] * self.VT.T[X[:, 1]]).sum(axis=1) +
                self.row_means[X[:, 0]] + self.col_means[X[:, 1]])

    def predict_single_user(self, work_ids, user_parameters):
        """
        Predict ratings for a single user.
        """
        mean, U = user_parameters
        return mean + U.dot(self.VT[:, work_ids]) + self.col_means[work_ids]

    def get_shortname(self):
        """
        Short name useful for logging output.
        """
        return f'svdknn-{self.nb_components}-{self.nb_neighbors}'
