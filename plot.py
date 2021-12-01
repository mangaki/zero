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
options = parser.parse_args()

fig, ax = plt.subplots(1, 1)

for filename in glob.glob('results/*.json'):
    title = re.match('results/(.*).json', filename).group(1)
    with open(filename) as f:
        model_metrics = json.load(f)
    if options.metric in model_metrics['train']:
        ax.plot(model_metrics['train'][options.metric], label=title)

ax.legend()
ax.set_title('Comparing Mangaki Zero algorithms on Movielens')
ax.set_ylabel(options.metric.upper())
ax.set_xlabel('Epochs')
fig.savefig('plot.png')
