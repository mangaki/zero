from distutils.core import setup


setup(
    name='Mangaki Zero',
    version='1.0',
    description='Recommendation algorithms',
    author='Jill-Jênn Vie',
    author_email='vie@jill-jenn.net',
    url='http://research.mangaki.fr',
    python_requires='>=3.4',
    install_requires=[
        'numpy>=1.13.3,<=1.18.2',
        'scipy>=1',
        'pandas',
    ],
    packages=['zero']
)
