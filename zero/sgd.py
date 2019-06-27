import numpy as np
from scipy.sparse import coo_matrix
import random
import logging
import time

from zero.recommendation_algorithm import (RecommendationAlgorithm,
                                           register_algorithm)


@register_algorithm('sgd')
class MangakiSGD(RecommendationAlgorithm):
    def __init__(self, nb_components=20, nb_iterations=10,
                 gamma=0.01, lambda_=0.1):
        super().__init__()
        self.nb_components = nb_components
        self.nb_iterations = nb_iterations
        self.gamma = gamma
        self.lambda_ = lambda_

    def fit(self, X, y):
        self.bias = np.random.random()
        self.bias_u = np.random.random(self.nb_users)
        self.bias_v = np.random.random(self.nb_works)
        self.U = np.random.random((self.nb_users, self.nb_components))
        self.V = np.random.random((self.nb_works, self.nb_components))
        for epoch in range(self.nb_iterations):
            step = 0
            dt = time.time()
            for (i, j), rating in zip(X, y):
                predicted_rating = self.predict_one(i, j)
                error = predicted_rating - rating
                self.bias -= self.gamma * error
                self.bias_u[i] -= self.gamma * (error +
                                                self.lambda_ * self.bias_u[i])
                self.bias_v[j] -= self.gamma * (error +
                                                self.lambda_ * self.bias_v[j])
                self.U[i] -= self.gamma * (error * self.V[j] +
                                           self.lambda_ * self.U[i])
                self.V[j] -= self.gamma * (error * self.U[i] +
                                           self.lambda_ * self.V[j])
                step += 1
            print('elapsed', time.time() - dt)
            self.compute_metrics()

    def fit_single_user(self, rated_works, ratings):
        bias_user = np.mean(ratings)
        feat_user = np.random.random(self.nb_components)
        for epoch in range(self.nb_iterations):
            for j, rating in zip(rated_works, ratings):
                pred = self.bias + bias_user + self.bias_v[j] + feat_user.dot(self.V[j])
                error = pred - rating
                bias_user -= self.gamma * (error + self.lambda_ * bias_user)
                feat_user -= self.gamma * (error * self.V[j] + self.lambda_ * feat_user)
        return bias_user, feat_user

    def predict_one(self, i, j):
        return (self.bias + self.bias_u[i] + self.bias_v[j] +
                self.U[i].dot(self.V[j]))

    def predict(self, X):
        y = []
        for user_id, work_id in X:
            y.append(self.predict_one(user_id, work_id))
        return np.array(y)

    def predict_single_user(self, work_ids, user_parameters):
        bias_user, feat_user = user_parameters
        return self.bias + bias_user + self.bias_v[work_ids] + self.V[work_ids].dot(feat_user)

    @property
    def is_serializable(self):
        return False  # Not yet, but easy to do

    def __str__(self):
        return '[SGD] NB_COMPONENTS = %d' % self.nb_components

    def get_shortname(self):
        return 'sgd-%d' % self.nb_components
