# Zero

Mangaki's recommendation algorithms.

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
