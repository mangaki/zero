"""
Mangaki sparse SVD with KNN.
Author: Jill-Jênn Vie, 2022
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
.   Then it computes the average embedding of k-nearest neighbors.
    '''
    def __init__(self, nb_components=20, nb_neighbors=5, is_weighted=True,
                 nb_iterations=None, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.U = None
        self.sigma = None
        self.VT = None
        self.nb_components = nb_components
        self.nb_neighbors = nb_neighbors
        self.is_weighted = is_weighted
        self.means = None
        self.knn = NearestNeighbors(n_neighbors=self.nb_neighbors)

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
        self.user_embeddings = self.U * self.sigma
        self.knn.fit(self.user_embeddings)

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

    def compute_average_embedding(self, query):
        dist, neighbor_ids = self.knn.kneighbors(query)
        if 0 in dist:  # Query is in the stored embeddings (predict)
            adj = self.knn.kneighbors_graph(query, self.nb_neighbors + 1,
                                            mode='distance')
        else:  # predict_single_user
            adj = self.knn.kneighbors_graph(query, mode='distance')
        if self.is_weighted:
            adj.data[adj.data > 0] = 1 / adj.data[adj.data > 0]
        else:
            adj.data[adj.data > 0] = np.ones_like(adj.data[adj.data > 0])
        answer = adj @ self.user_embeddings / adj.sum(axis=1)
        return answer

    def predict(self, X):
        """
        Predict ratings for user, item pairs.
        """
        pred_user_ids = list(set(X[:, 0]))
        averaged_embeddings = self.compute_average_embedding(
            self.user_embeddings[pred_user_ids])
        averaged_embedding = np.zeros_like(self.user_embeddings)
        averaged_embedding[pred_user_ids] = averaged_embeddings            
        return ((averaged_embedding[X[:, 0]] * self.VT.T[X[:, 1]]).sum(axis=1) +
                self.row_means[X[:, 0]] + self.col_means[X[:, 1]])

    def predict_single_user(self, work_ids, user_parameters):
        """
        Predict ratings for a single user.
        """
        mean, U = user_parameters
        average_embedding = self.compute_average_embedding(
            U.reshape(1, -1)).A1  # Make it a flattened embedding no matter what
        return (mean + average_embedding.dot(self.VT[:, work_ids]) +
                self.col_means[work_ids])

    def get_shortname(self):
        """
        Short name useful for logging output.
        """
        suffix = '-weight' if self.is_weighted else ''
        return f'svdknn-{self.nb_components}-{self.nb_neighbors}{suffix}'
