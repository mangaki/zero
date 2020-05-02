# Zero

[![Mangaki Zero's CI status](https://github.com/mangaki/zero/workflows/CI/badge.svg)](https://mangaki/zero/actions)
[![Mangaki Zero's code coverage](https://codecov.io/gh/mangaki/zero/branch/master/graph/badge.svg)](https://codecov.io/gh/mangaki/zero)



Mangaki's recommendation algorithms.

It is tested on Python 3.6, 3.7 and 3.8 over OpenBLAS LP64 & MKL.

## Usage

Most models have the following routines:

    from zero.als import MangakiALS
    model = MangakiALS(nb_components=10)
    model.fit(X, y)
    model.predict(X)

There are a couple of other methods that can be used for online fit, say `model.predict_single_user(work_ids, user_parameters)`.

To run k-fold cross-validation, do:

    python compare.py <path/to/dataset>

## Results

### Mangaki data

![Comparing on Mangaki](results/mangaki.png)

### Movielens data

![Comparing on Movielens](results/movielens.png)

Feel free to use. Under GPLv3 license.
