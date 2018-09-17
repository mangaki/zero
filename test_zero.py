from zero.recommendation_algorithm import RecommendationAlgorithm
from zero.knn import normalize
from scipy.sparse import csr_matrix, diags
from scipy.sparse.linalg import norm
import numpy as np
import unittest
import logging
import os


ML_SNAPSHOT_ROOT_TEST = '/tmp/test_algo'


class AlgoTest(unittest.TestCase):
    def setUp(self):
        self.nb_users = 5
        self.nb_works = 10
        self.nb_tags = 2
        self.U = np.random.random((self.nb_users, 2))
        self.VT = np.random.random((2, self.nb_works))
        self.T = np.random.random((self.nb_works, self.nb_tags))
        self.M = self.U.dot(self.VT)
        train_user_ids = [1, 2, 3, 3]
        train_work_ids = [0, 1, 0, 1]
        self.X_train = np.column_stack((train_user_ids, train_work_ids))
        self.y_train = self.M[train_user_ids, train_work_ids]
        test_user_ids = [1, 2]
        test_work_ids = [1, 0]
        self.X_test = np.column_stack((test_user_ids, test_work_ids))
        self.y_test = self.M[test_user_ids, test_work_ids]

        if not os.path.exists(ML_SNAPSHOT_ROOT_TEST):
            os.makedirs(ML_SNAPSHOT_ROOT_TEST)

    def test_normalize(self):
        X = csr_matrix(np.random.random((5, 2)))
        X_normalized = normalize(X)
        norms_normalized = norm(X_normalized, axis=1)
        for entry in norms_normalized:
            self.assertLessEqual(abs(entry - 1), 1e-6)

    def test_fit_predict(self):
        for algo_name in RecommendationAlgorithm.list_available_algorithms():
            algo = RecommendationAlgorithm.instantiate_algorithm(algo_name)
            algo.set_parameters(self.nb_users, self.nb_works)
            if algo_name in {'balse', 'fma', 'gbr', 'lasso', 'xals'}:
                algo.nb_tags = self.nb_tags
                algo.T = self.T
            if algo_name == 'svd':
                algo.U = self.U
                algo.sigma = np.ones(2)
                algo.VT = self.VT
                algo.means = np.zeros(self.nb_users)
            algo.X_train = self.X_train
            algo.y_train = self.y_train
            algo.X_test = self.X_test
            algo.y_test = self.y_test
            if algo_name != 'svd':  # To avoid loading sklearn just for that
                algo.fit(self.X_train, self.y_train)
            if algo_name in {'als', 'knn', 'svd'}:
                user_parameters = algo.fit_single_user([1], [2])
                y_pred = algo.predict_single_user(list(range(self.nb_works)),
                                                  user_parameters)
                self.assertEqual(len(y_pred.shape), 1)
            if algo.is_serializable:
                algo.save(ML_SNAPSHOT_ROOT_TEST)
                algo.load(ML_SNAPSHOT_ROOT_TEST)
            y_pred = algo.predict(self.X_test)
            if algo.is_serializable:
                algo.delete_snapshot()
            logging.debug('rmse=%.3f algo=%s',
                          algo.compute_rmse(y_pred, self.y_test), algo_name)

    def tearDown(self):
        os.removedirs(ML_SNAPSHOT_ROOT_TEST)
