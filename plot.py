import json
import matplotlib.pyplot as plt
import glob
import re


fig, ax = plt.subplots(1, 1)

for filename in glob.glob('results/*.json'):
    title = re.match('results/(.*).json', filename).group(1)
    with open(filename) as f:
        model_metrics = json.load(f)
    ax.plot(model_metrics['test']['rmse'], label=title)

ax.legend()
ax.set_title('Comparing Mangaki Zero algorithms on Movielens')
ax.set_ylabel('RMSE')
ax.set_xlabel('Epochs')
fig.savefig('plot.png')
