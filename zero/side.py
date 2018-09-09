from scipy.sparse import load_npz, issparse, identity
import numpy as np
import os.path


class SideInformation:
    def __init__(self, T=None, perform_scaling=True, with_mean=False):
        self.T = T
        self.nb_tags = None
        self.perform_scaling = perform_scaling
        self.with_mean = with_mean
        self.load()

    def load(self):
        # Load in CSC format if no matrix provided.
        if self.T is None:
            tags_path = os.path.join('tags', 'tag-matrix.npz')
            if os.path.isfile(tags_path):
                self.T = load_npz(tags_path)
            else:
                self.T = identity(0)
        _, self.nb_tags = self.T.shape
