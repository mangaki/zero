Quick start
===========

Install
-------

::

    python -m venv venv
    source venv/bin/activate
    pip install -r requirements.txt

Usage
-----

To run cross-validation:

1. Download a dataset like `Movielens 100k <http://files.grouplens.org/datasets/movielens/ml-latest-small.zip>`_
2. Ensure the columns are named :code:`user,item,rating`:

==== ==== ======
user item rating
==== ==== ======
3    5    4.5
==== ==== ======

For example, here, user 3 gave 4.5 stars to item 5.

3. Then run: ::

    python compare.py <path/to/dataset>

You can tweak the :code:`experiments/default.json` file to compare other models.

Custom usage
------------

Most models have the following routines: ::

    from zero.als import MangakiALS
    model = MangakiALS(nb_components=10)
    model.fit(X, y)
    model.predict(X)

where:

- *X* is a numpy array of size `nb_samples` x 2 (first column: user ID, second column: item ID)
- and *y* is the column of ratings.

There are a couple of other methods that can be used for online fit, say :code:`model.predict_single_user(work_ids, user_parameters)`.

See `zero.py <_modules/zero/zero.html#MangakiZero>`_ as an example of dumb baseline (only predicts zeroes) to start from. See `svd2.py <_modules/zero/svd2.html#MangakiSVD2>`_ for a more advanced example.

Results
-------

Mangaki data
::::::::::::

.. image:: ../results/mangaki.png
   :alt: Comparing on Mangaki

Movielens data
::::::::::::::

.. image:: ../results/movielens.png
   :alt: Comparing on Movielens

Feel free to use. Under MIT license.
