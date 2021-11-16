import argparse
import glob
import json
import matplotlib.pyplot as plt
import re

parser = argparse.ArgumentParser(description='Plot results from compare.py')
parser.add_argument('-em', '--eval-metric',
                    dest='metric',
                    help='Plot a specific metric',
                    default='rmse')
options = vars(parser.parse_args())

metric = options.get('metric')

fig, ax = plt.subplots(1, 1)

for filename in glob.glob('results/*.json'):
    title = re.match('results/(.*).json', filename).group(1)
    if title[:3] == "svd":
        continue
    with open(filename) as f:
        model_metrics = json.load(f)
    if metric in model_metrics['test']:
        ax.plot(model_metrics['test'][metric], label=title)

ax.legend()
ax.set_title('Comparing Mangaki Zero algorithms on Movielens')
ax.set_ylabel(metric.upper())
ax.set_xlabel('Epochs')
fig.savefig('plot.png')
