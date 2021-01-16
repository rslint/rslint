import numpy as np
import matplotlib.pyplot as plt
import os
import pandas

NANOS = 1_000_000

current_directory = os.path.dirname(os.path.realpath(__file__))
runtime_dir = os.path.join(
    current_directory, "../target/criterion/engine262/batched analysis times")
throughput_dir = os.path.join(
    current_directory, "../target/criterion/engine262/batched analysis throughput")

# { "num_threads": { "num_elements": { "throughput": float, "runtime": float }}}
points = {}

for dir in os.listdir(runtime_dir):
    if dir == "report":
        continue

    dir = os.path.join(runtime_dir, dir)
    path = os.path.join(dir, "new/raw.csv")

    data = pandas.read_csv(path)

    averages = [(tot / iters) / NANOS for tot,
                iters in zip(data.sample_measured_value, data.iteration_count)]
    samples = len(list(data.sample_measured_value))
    points[str(data.value[0])] = {
        "runtime": float(sum(averages) / samples),
    }


for dir in os.listdir(throughput_dir):
    if dir == "report":
        continue

    dir = os.path.join(throughput_dir, dir)
    path = os.path.join(dir, "new/raw.csv")

    data = pandas.read_csv(path)

    averages = [through / ((tot / iters) / NANOS) for through, tot,
                iters in zip(data.throughput_num, data.sample_measured_value, data.iteration_count)]
    samples = len(list(data.sample_measured_value))
    points[str(data.value[0])].update({
        "throughput": float(sum(averages) / samples),
    })

figure, axes = plt.subplots(1, 2)
runtime_x_max, throughput_x_max, y_max = 0, 0, 0
runtime_points, throughput_points = {}, {}

for num_threads in points:
    threads, files = num_threads.split(", ")
    files = int(files.rstrip(" files").strip())
    if files > y_max:
        y_max = files

    runtime = points[num_threads]["runtime"]
    if runtime > runtime_x_max:
        runtime_x_max = runtime

    if threads in runtime_points:
        runtime_points[threads].append((runtime, files))
    else:
        runtime_points[threads] = [(runtime, files)]

    throughput = points[num_threads]["throughput"]
    if throughput > throughput_x_max:
        throughput_x_max = throughput

    if threads in throughput_points:
        throughput_points[threads].append((throughput, files))
    else:
        throughput_points[threads] = [(throughput, files)]

axes[0].set_title("Runtime")
axes[0].set_ylabel("milliseconds")
axes[0].set_xlabel("files in batch")
for threads in runtime_points:
    y, x = zip(*runtime_points[threads])
    axes[0].scatter(list(x), list(y), label=threads, marker='x')
    # axes[0].plot(np.polyfit(list(x), list(y), 5), "black")

    coeffs = np.polyfit(list(x), list(y), 10)
    x2 = np.arange(min(x), max(x), .01)
    y2 = np.polyval(coeffs, x2)
    axes[0].plot(x2, y2, "black")

axes[1].set_title("Throughput")
axes[1].set_ylabel("records per millisecond")
axes[1].set_xlabel("files in batch")
for threads in throughput_points:
    y, x = zip(*throughput_points[threads])
    axes[1].scatter(list(x), list(y), label=threads, marker='x')

    coeffs = np.polyfit(list(x), list(y), 10)
    x2 = np.arange(min(x), max(x), .01)
    y2 = np.polyval(coeffs, x2)
    axes[1].plot(x2, y2, "black")

axes[1].legend()
axes[0].legend()
plt.show()
