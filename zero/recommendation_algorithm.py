from zero.side import SideInformation
from zero.chrono import Chrono
from collections import defaultdict
from itertools import product
import numpy as np
import pickle
import os.path
import logging


class RecommendationAlgorithmFactory:
    def __init__(self):
        self.algorithm_registry = {}
        self.algorithm_factory = {}
        self.logger = logging.getLogger(__name__ + '.' +
                                        self.__class__.__name__)
        self.logger.setLevel(logging.INFO)
        self.initialized = False
        self.size = 0

    def initialize(self):
        # FIXME: make it less complicated and go for a commonly used design
        # pattern.
        # Behind the hood, it's called in `utils.__init__.py` which triggers
        # the `algos.__init__.py`
        # which in turn triggers registration on this instance.
        # Then, once it reach `recommendation_algorithm` file, it's good to go.
        self.logger.debug('Recommendation algorithm factory initialized.'
                          '{} algorithms available in the factory.'
                          .format(len(self.algorithm_registry)))
        self.initialized = True

    def register(self, name, klass, default_kwargs):
        self.algorithm_registry[name] = klass
        self.algorithm_factory[name] = default_kwargs
        self.logger.debug('Registered {} as a recommendation algorithm'.format(
                          name))


class RecommendationAlgorithm:
    factory = RecommendationAlgorithmFactory()

    def __init__(self, metrics=None, verbose_level=1):
        self.verbose_level = verbose_level
        self.logger = logging.getLogger(__name__ + '.' +
                                        self.__class__.__name__)
        self.logger.setLevel(logging.DEBUG)
        self.chrono = Chrono(self.verbose_level)
        self.nb_users = None
        self.nb_works = None
        self.size = 0  # For backup files
        if metrics is None:
            metrics = ['rmse']
        self.metrics = {category: {metric: [] for metric in metrics}
                        for category in {'train', 'test'}}
        self.dataset = None
        self.X_train = None
        self.y_train = None
        self.X_test = None
        self.y_test = None

    def get_backup_path(self, folder, filename):
        if not self.is_serializable:
            raise NotImplementedError
        if filename is None:
            filename = '%s.pickle' % self.get_shortname()
        return os.path.join(folder, filename)

    # def has_backup(self, filename=None):
    #     if filename is None:
    #         filename = self.get_backup_filename()
    #     return os.path.isfile(self.get_backup_path(filename))

    @property
    def is_serializable(self):
        return False

    def save(self, folder, filename=None):
        self.backup_path = self.get_backup_path(folder, filename)
        with open(self.backup_path, 'wb') as f:
            pickle.dump(self.__dict__, f, pickle.HIGHEST_PROTOCOL)
        self.size = os.path.getsize(self.backup_path)  # In bytes

    def load(self, folder, filename=None):
        """
        This function raises FileNotFoundException if no backup exists.
        """
        self.backup_path = self.get_backup_path(folder, filename)
        with open(self.backup_path, 'rb') as f:
            backup = pickle.load(f)
        self.__dict__.update(backup)

    def delete_snapshot(self):
        os.remove(self.backup_path)

    def recommend(self, user_ids, extra_users_parameters=None, item_ids=None,
                  k=None, method='mean'):
        """
        Recommend :math:`k` items to a group of users.

        :param user_ids: the users that are in the dataset of this algorithm.
        :param extra_users_parameters: the parameters for users that weren't.
        :param item_ids: a subset of items. If is it None, then it is all items.
        :param k: the number of items to recommend, if None then it is all items.
        :param method: a way to combine the predictions. By default it is mean.
        :returns: a numpy array with two columns, `item_id` and recommendation score
        :complexity: :math:`O(N + K \log K)`
        """
        if item_ids is None:
            item_ids = np.arange(self.nb_works)
        n = len(item_ids)
        if k is None:
            k = n
        k = min(n, k)
        if user_ids is not None and len(user_ids):
            X = np.array(list(product(user_ids, item_ids)))
            cache_pred = self.predict(X).reshape(len(user_ids), -1)
        else:
            cache_pred = np.zeros((0, len(item_ids)))
        if extra_users_parameters is not None and len(extra_users_parameters):
            extra_pred = np.array([
                self.predict_single_user(item_ids, parameters)
                for parameters in extra_users_parameters
            ])
        else:
            extra_pred = np.zeros((0, len(item_ids)))
        pred = np.concatenate((cache_pred, extra_pred), axis=0)
        if method == 'mean':
            combined_pred = pred.mean(axis=0)
            indices = np.argpartition(combined_pred, n - k)[-k:]
            results = np.empty(k, dtype=[('item_id', int), ('score', combined_pred.dtype)])
            results['item_id'] = indices
            results['score'] = combined_pred[indices]
            results.sort(order='score')
            return results[::-1]
        else:
            raise NotImplementedError

    def load_tags(self, T=None, perform_scaling=True, with_mean=False):
        side = SideInformation(T, perform_scaling, with_mean)
        self.nb_tags = side.nb_tags
        self.T = side.T

    def set_parameters(self, nb_users, nb_works):
        self.nb_users = nb_users
        self.nb_works = nb_works

    def get_shortname(self):
        return 'algo'

    @staticmethod
    def compute_rmse(y_pred, y_true):
        return np.power(y_true - y_pred, 2).mean() ** 0.5

    @staticmethod
    def compute_mae(y_pred, y_true):
        return np.abs(y_true - y_pred).mean()

    def get_ranked_gains(self, y_pred, y_true):
        return y_true[np.argsort(y_pred)[::-1]]

    def compute_dcg(self, y_pred, y_true):
        '''
        Computes the discounted cumulative gain as stated in:
        https://gist.github.com/bwhite/3726239
        '''
        ranked_gains = self.get_ranked_gains(y_pred, y_true)
        return self.dcg_at_k(ranked_gains, 100)

    def compute_ndcg(self, y_pred, y_true):
        ranked_gains = self.get_ranked_gains(y_pred, y_true)
        return self.ndcg_at_k(ranked_gains, 100)

    def dcg_at_k(self, r, k):
        r = np.asfarray(r)[:k]
        if r.size:
            return np.sum(np.subtract(np.power(2, r), 1) /
                          np.log2(np.arange(2, r.size + 2)))
        return 0.

    def ndcg_at_k(self, r, k):
        idcg = self.dcg_at_k(sorted(r, reverse=True), k)
        if not idcg:
            return 0.
        return self.dcg_at_k(r, k) / idcg

    def compute_metrics(self):
        for mode in ('train', 'test'):
            X = getattr(self, 'X_{}'.format(mode))
            if X is not None:
                y_true = getattr(self, 'y_{}'.format(mode))
                y_pred = self.predict(X)
                log = mode
                for metric in self.metrics['test'].keys():
                    compute_method = getattr(self, 'compute_{}'.format(metric))
                    value = compute_method(y_pred, y_true)
                    self.metrics[mode][metric].append(value)
                    log += " {}={:.6f}".format(metric, value)
                self.logger.info(log)

    @staticmethod
    def available_evaluation_metrics():
        return ['rmse', 'mae', 'dcg', 'ndcg']

    @classmethod
    def register_algorithm(cls, name, klass, default_kwargs=None):
        cls.factory.register(name, klass, default_kwargs)

    @classmethod
    def list_available_algorithms(cls):
        return list(cls.factory.algorithm_registry.keys())

    @classmethod
    def instantiate_algorithm(cls, name):
        klass = cls.factory.algorithm_registry.get(name)
        default_kwargs = cls.factory.algorithm_factory.get(name) or {}
        if not klass:
            raise KeyError('No algorithm named "{}" in the registry! Did you '
                           'forget a @register_algorithm? A typo?'
                           .format(name))

        return klass(**default_kwargs)

    def __str__(self):
        return '[%s]' % self.get_shortname().upper()


def register_algorithm(algorithm_name, default_kwargs=None):
    if default_kwargs is None:
        default_kwargs = {}

    def decorator(cls):
        RecommendationAlgorithm.register_algorithm(algorithm_name, cls,
                                                   default_kwargs)
        return cls
    return decorator
