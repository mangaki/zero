from distutils.core import setup


setup(
    name='mangaki-zero',
    version='1.0',
    description='Recommendation algorithms',
    author='Jill-Jênn Vie',
    author_email='vie@jill-jenn.net',
    url='https://research.mangaki.fr',
    python_requires='>=3.4',
    install_requires=[
        'numpy>=1.13.3,<=1.14.5',
        'scipy>=1',
        'pandas',
    ],
    packages=['zero']
)
