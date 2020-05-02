from zero.recommendation_algorithm import RecommendationAlgorithm
from zero.knn import normalize
from scipy.sparse import csr_matrix, diags
from scipy.sparse.linalg import norm
import numpy as np
import unittest
import logging
import os
import pytest


# Test normalization function.
def test_normalize():
    X = csr_matrix(np.random.random((5, 2)))
    X_normalized = normalize(X)
    norms_normalized = norm(X_normalized, axis=1)
    for entry in norms_normalized:
        assert abs(entry - 1) <= 1e-6

@pytest.mark.parametrize(
        ("algo_name", "nb_users", "nb_works", "nb_tags"),
        [(name, 10, 10, 10) for name in RecommendationAlgorithm.list_available_algorithms()])
def test_fit_predict(
        algo_name,
        nb_users,
        nb_works,
        nb_tags,
        tmp_path):
    # set up some variables.
    U = np.random.random((nb_users, 2))
    VT = np.random.random((2, nb_works))
    T = np.random.random((nb_works, nb_tags))
    M = U.dot(VT)

    train_user_ids = [1, 2, 3, 3]
    train_work_ids = [0, 1, 0, 1]
    X_train = np.column_stack((train_user_ids, train_work_ids))
    y_train = M[train_user_ids, train_work_ids]
    test_user_ids = [1, 2]
    test_work_ids = [1, 0]
    X_test = np.column_stack((test_user_ids, test_work_ids))
    y_test = M[test_user_ids, test_work_ids]

    print('Testing algorithm', algo_name)
    algo = RecommendationAlgorithm.instantiate_algorithm(algo_name)
    algo.set_parameters(nb_users, nb_works)
    if algo_name in {'balse', 'fma', 'gbr', 'lasso', 'xals'}:
        algo.nb_tags = nb_tags
        algo.T = T
    if algo_name == 'svd':
        algo.U = U
        algo.sigma = np.ones(2)
        algo.VT = VT
        algo.means = np.zeros(nb_users)
    algo.X_train = X_train
    algo.y_train = y_train
    algo.X_test = X_test
    algo.y_test = y_test
    if algo_name != 'svd':  # To avoid loading sklearn just for that
        algo.fit(X_train, y_train)
    if algo_name in {'als', 'knn', 'sgd', 'svd'}:
        user_parameters = algo.fit_single_user([1], [2])
        y_pred = algo.predict_single_user(list(range(nb_works)),
                                          user_parameters)
        assert len(y_pred.shape) == 1
    if algo.is_serializable:
        tmp_path.mkdir(exist_ok=True)
        algo.save(tmp_path)
        algo.load(tmp_path)
    y_pred = algo.predict(X_test)
    if algo.is_serializable:
        algo.delete_snapshot()
    logging.debug('rmse=%.3f algo=%s',
                  algo.compute_rmse(y_pred, y_test), algo_name)
