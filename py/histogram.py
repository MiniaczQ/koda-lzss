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
    :type file: io.BufferedReader
    :raises RuntimeError: When header is invalid for any reason.
    :return: tuple of the following format: ((width, height), maxValue).
    :rtype: tuple[tuple[int, int], int]
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


def read_file(path: str) -> tuple[np.ndarray, tuple[int, int], int]:
    """
    Parse raw binary data into a NumPy array.

    :param path: String path to the file.
    :type path: str
    :return: Tuple of NumPy array holding data, dataset dimensions and maximum possible value.
    :rtype: tuple[np.ndarray, int]
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
    :type sums: list[int]
    :param total: Total sample count.
    :type total: int
    :return: Entropy value.
    :rtype: float
    """
    entropy = 0
    for sum in sums:
        probability = sum / total
        if probability != 0:
            entropy -= probability * math.log2(probability)
    return entropy


def calculate_n_order(data: np.ndarray, order_number: int) -> tuple[list[int], int]:
    """
    TODO: Summary

    :param data: Input data.
    :type data: np.ndarray
    :param order_number: Order of block source.
    :type order_number: int
    :return: TODO: Return summary
    :rtype: tuple[list[int], int]
    """
    block_pairs: dict[int, int] = {}
    total_pairs = len(data) - order_number
    for i in range(total_pairs):
        key = data[i:i+order_number]
        block_pairs[key] = block_pairs[key] + 1 if key in block_pairs else 1
    return list(block_pairs.values()), total_pairs


def process_file(dir: str, name: str, axis: plt.Axes) -> tuple[float, float, float]:
    """
    Given relative path to the file, process its content, draw a histogram and calculate entropy.

    :param dir: String path to the directory.
    :type dir: str
    :param name: String name of the file (also subplot name).
    :type name: str
    :param axis: PyPlot axis to draw to.
    :type axis: plt.Axes
    :return: Calculated entropy of the dataset.
    :rtype: tuple[float, float, float]
    """
    data, dimensions, max = read_file(dir + name)
    # Save bars from the histogram
    _, _, bars = axis.hist(data, bins=max+1, range=(0, max))
    axis.set_title(name)
    axis.set_xlim(0, max)
    # Extract bar values and calculate entropy
    sums = [bar.get_height() for bar in bars]
    entropy = calculate_entropy(sums, dimensions[0] * dimensions[1])
    # Calculate second order entropy
    second_sums, second_total = calculate_n_order(data, 2)
    second_entropy = calculate_entropy(second_sums, second_total)
    # Calculate third order entropy
    third_sums, third_total = calculate_n_order(data, 3)
    third_entropy = calculate_entropy(third_sums, third_total)
    return entropy, second_entropy, third_entropy


def process_folder(path: str) -> list[tuple[str, float]]:
    """
    Given relative path to a directory, process all files inside and create a rectangular plot grid with histograms.

    :param path: String path to the directory.
    :type path: str
    :return: List of pairs (file name, entropy).
    :rtype: list[tuple[str, float]]
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
        entropy_set = process_file(path, file, axes[i // grid_shape[1]][i % grid_shape[1]])
        entropies.append((file, entropy_set))
    return entropies


def create_hist(path: str, file_name: str) -> None:
    """
    Create and save a  histogram for a file in the given folder.

    :param path: String path to the directory.
    :type path: str
    :param file_name: File name.
    :type file_name: str
    """
    data, dimensions, max = read_file(os.path.join(path, file_name))
    plt.figure()
    plt.hist(data, bins=max, range=(0, max))
    plt.title(file_name)
    save_name =  os.path.join("py", "histograms", os.path.splitext(file_name)[0] + ".png")
    plt.savefig(save_name)
    

def hist_for_files(path: str) -> None:
    """
    Create and save a separate histogram for every file in the folder.

    :param path: String path to the directory.
    :type path: str
    """
    # Obtain a list of all files in a directory.
    files = os.listdir(os.path.dirname(__file__) + "/" + path)
    # Process each file sequentially and show and save the histogram for it.
    for i, file in enumerate(files):
        print(f"Generating histogram and saving it #{i + 1}: {file}...")
        create_hist(path, file)


if __name__ == "__main__":
    if len(sys.argv) > 1:
        for filename in sys.argv[1:]:
            axes = plt.axes()
            entropy = process_file('', filename, axes)
            print(f'Entropy for file {filename}: {entropy:.3f}')
            plt.show()
        exit()
    results = []
    dirs = ["../data/txt/", "../data/img/", "../data/random/"]
    ## Calculate normal, second- and third- block order entropy values for files
    for dir in dirs:
        print(f"Processing folder {dir}...")
        entropies = process_folder(dir)
        results.extend(entropies)
    print("Entropy values:")
    for file, entropy in results:
        print(f"File {file}: \tEntropy {round(entropy[0],3)} \t| Entropy second {round(entropy[1],3)} \t| Entropy third {round(entropy[2],3)}")
    ## Create separate histograms for each file
    #for dir in dirs:
    #    hist_for_files(dir)
    plt.show()
