from zero.side import SideInformation
from zero.chrono import Chrono
from collections import defaultdict
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

    def __init__(self, verbose_level=1):
        self.verbose_level = verbose_level
        self.chrono = Chrono(self.verbose_level)
        self.nb_users = None
        self.nb_works = None
        self.size = 0  # For backup files
        self.metrics = {category: defaultdict(list)
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
        if self.X_train is not None:
            y_train_pred = self.predict(self.X_train)
            train_rmse = self.compute_rmse(self.y_train, y_train_pred)
            self.metrics['train']['rmse'].append(train_rmse)
            logging.warning('Train RMSE=%f', train_rmse)
        if self.X_test is not None:
            y_test_pred = self.predict(self.X_test)
            test_rmse = self.compute_rmse(self.y_test, y_test_pred)
            self.metrics['test']['rmse'].append(test_rmse)
            logging.warning('Test RMSE=%f', test_rmse)

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
