# Zero

[![Mangaki Zero's CI status](https://github.com/mangaki/zero/workflows/CI/badge.svg)](https://github.com/mangaki/zero/actions)
[![Mangaki Zero's code coverage](https://codecov.io/gh/mangaki/zero/branch/master/graph/badge.svg)](https://codecov.io/gh/mangaki/zero)
[API documentation](https://mangaki.github.io/zero/)

Mangaki's recommendation algorithms.

They are tested on Python 3.9, 3.10 over OpenBLAS LP64. Intel MKL is broken for now, see <https://github.com/mangaki/zero/issues/25>.

## Install

    uv sync

Or

	poetry install

## Usage

To run cross-validation:

1. Download a dataset like [Movielens 100k](http://files.grouplens.org/datasets/movielens/ml-latest-small.zip).
2. Ensure the columns are named `user,item,rating`:

user | item | rating
--- | --- | ---
3 | 5 | 4.5

For example, here, user 3 gave 4.5 stars to item 5.

3. Then run:

    python compare.py <path/to/dataset>

You can tweak the `experiments/default.json` file to compare other models.

## Tests

This assumes you installed the dependencies of the project.

- To run tests for the Python Zero module: `py.test zero`
- To run tests for the Aggregation module: `py.test aggregation`

## Custom usage

Most models have the following routines:

    from zero.als import MangakiALS
    model = MangakiALS(nb_components=10)
    model.fit(X, y)
    model.predict(X)

where:

- *X* is a numpy array of size `nb_samples` x 2
(first column: user ID, second column: item ID)
- and *y* is the column of ratings.

There are a couple of other methods that can be used for online fit, say `model.predict_single_user(work_ids, user_parameters)`.

See [zero.py](zero/zero.py) as an example of dumb baseline (only predicts zeroes) to start from.

## Secure aggregation module usage

Install this module with the extra `secure-aggregation`, i.e. `pip install mangaki-zero[secure-aggregation]` or compile the module in `aggregation/`, this only requires a stable Rust compiler (CI tests are performed against Rust stable, beta and nightlies.) and [maturin](https://github.com/PyO3/maturin/).

Then, you can follow the docs there: <https://mangaki.github.io/zero/>

## Results

### Mangaki data

![Comparing on Mangaki](results/mangaki.png)

### Movielens data

![Comparing on Movielens](results/movielens.png)

Feel free to use. Under MIT license.
