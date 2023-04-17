import io
import math
import os
import sys
import matplotlib.pyplot as plt
import numpy as np

def validate_header(file: io.BufferedReader) -> tuple[tuple[int, int], int]:
    """
    Validate and extract data from a PGM header.

    :param file: Data source.
    :returns: tuple of the following format: ((width, height), maxValue)
    :raises RuntimeError: When header is invalid for any reason
    """
    # Read the first line - should always be "P5".
    type = file.readline()[:-1]
    if type != b'P5':
        raise RuntimeError("File isn't a valid type 5 PGM file!")

    # Read the second line - it may be a comment (in this case, skip), or space-separated dataset dimensions.
    dimensions = file.readline()[:-1].decode()
    if dimensions[0] == '#':
        dimensions = file.readline()[:-1].decode()
    dimensions = dimensions.split(' ')

    # Validate dimensions.
    if\
        len(dimensions) != 2\
        or not dimensions[0].isnumeric() or int(dimensions[0]) <= 0\
        or not dimensions[1].isnumeric() or int(dimensions[1]) <= 0:
            raise RuntimeError("File has incorrect dimensions set!")

    # Read the third (fourth with a comment) line - maximum possible value for a dataset sample.
    max = file.readline()[:-1].decode()
    if not max.isnumeric() or int(max) <= 0:
        raise RuntimeError("File has incorrect maximum value set!")

    return (tuple([int(x) for x in dimensions]), int(max))


def read_file(path: str) -> tuple[np.ndarray, int]:
    """
    Parse raw binary data into a NumPy array.

    :param path: String path to the file.
    :returns: Tuple of NumPy array holding data, dataset dimensions and maximum possible value.
    """
    data = None
    with open(os.path.dirname(__file__) + "/" + path, "rb") as file:
        # Validate header, extract dimension and maximum value information.
        dimensions, max = validate_header(file)
        # Continue reading from the file, raw data is unsigned 8 bit integers.
        data = np.fromfile(file, dtype=np.dtype('B'))

    return data, dimensions, max


def calculate_entropy(sums: list[int], total: int) -> float:
    """
    Calculate entropy of dataset processed into a histogram.

    :param sums: Counts of specific sample values.
    :param total: Total sample count.
    :returns: Entropy value.
    """
    entropy = 0
    for sum in sums:
        probability = sum / total
        if probability != 0:
            entropy -= probability * math.log2(probability)
    
    return entropy


def process_file(dir: str, name: str, axis: plt.Axes) -> float:
    """
    Given relative path to the file, process its content, draw a histogram and calculate entropy.

    :param dir: String path to the directory.
    :param name: String name of the file (also subplot name).
    :param axis: PyPlot axis to draw to.
    :returns: Calculated entropy of the dataset.
    """
    data, dimensions, max = read_file(dir + name)
    # Save bars from the histogram
    _, _, bars = axis.hist(data, bins=max+1, range=(0, max))
    axis.set_title(name)
    axis.set_xlim(0, max)
    # Extract bar values and calculate entropy
    sums = [bar.get_height() for bar in bars]
    entropy = calculate_entropy(sums, dimensions[0] * dimensions[1])

    return entropy


def process_folder(path: str) -> list[tuple[str, float]]:
    """
    Given relative path to a directory, process all files inside and create a rectangular plot grid with histograms.

    :param path: String path to the directory.
    :returns: List of pairs (file name, entropy).
    """
    entropies = []
    # Obtain a list of all files in a directory.
    files = os.listdir(os.path.dirname(__file__) + "/" + path)
    # Calculate an optimal (or rather, good enough) grid shape for axes.
    grid_shape = math.ceil(math.sqrt(len(files))), round(math.sqrt(len(files)))
    # Create figure and axes of given shape.
    fig, axes = plt.subplots(*grid_shape)
    # Process each file sequentially and assign its histogram to an axis.
    for file, i in zip(files, range(len(files))):
        print(f"Processing file #{i + 1}: {file}...")
        entropy = process_file(path, file, axes[i // grid_shape[1]][i % grid_shape[1]])
        entropies.append((file, entropy))
    
    return entropies


if __name__ == "__main__":
    if len(sys.argv) > 1:
        for filename in sys.argv[1:]:
            axes = plt.axes()
            entropy = process_file('', filename, axes)
            print(f'Entropy for file {filename}: {entropy:.3f}')
            plt.show()
        exit()
    results = []
    dirs = ["../data/txt/", "../data/img/"]
    for dir in dirs:
        print(f"Processing folder {dir}...")
        entropies = process_folder(dir)
        results.extend(entropies)
    
    print("Entropy values:")
    for file, entropy in results:
        print(f"File {file}: {entropy}")

    plt.show()
