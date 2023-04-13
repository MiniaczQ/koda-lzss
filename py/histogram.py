import numpy as np
import matplotlib.pyplot as plt
import os
import io
import math
from typing import Tuple


def validate_header(file: io.BufferedReader) -> Tuple[Tuple[int, int], int]:
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

    return (tuple(dimensions), int(max))


def read_file(path: str) -> Tuple[np.ndarray, int]:
    """
    Parse raw binary data into a NumPy array.

    :param path: String path to the file.
    :returns: Pair of NumPy array holding data and maximum possible value.
    """
    data = None
    with open(os.path.dirname(__file__) + "/" + path, "rb") as file:
        # Validate header, extract dimension and maximum value information.
        dimensions, max = validate_header(file)
        # Continue reading from the file, raw data is unsigned 8 bit integers.
        data = np.fromfile(file, dtype=np.dtype('B'))

    return data, max


def process_file(dir: str, name: str, axis: plt.Axes) -> None:
    """
    Given relative path to the file, process its content and draw a histogram.

    :param dir: String path to the directory.
    :param name: String name of the file (also subplot name).
    :param axis: PyPlot axis to draw to.
    :returns: None.
    """
    data, max = read_file(dir + name)
    axis.hist(data, bins=max, range=(0, max))
    axis.set_title(name)
    axis.set_xlim(0, max)


def process_folder(path: str) -> None:
    """
    Given relative path to a directory, process all files inside and create a rectangular plot grid with histograms.

    :param path: String path to the directory.
    :returns: None.
    """
    # Obtain a list of all files in a directory.
    files = os.listdir(os.path.dirname(__file__) + "/" + path)
    # Calculate an optimal (or rather, good enough) grid shape for axes.
    grid_shape = math.ceil(math.sqrt(len(files))), round(math.sqrt(len(files)))
    # Create figure and axes of given shape.
    fig, axes = plt.subplots(*grid_shape)
    # Process each file sequentially and assign its histogram to an axis.
    for file, i in zip(files, range(len(files))):
        print(f"Processing file #{i + 1}: {file}...")
        process_file(path, file, axes[i // grid_shape[1]][i % grid_shape[1]])


if __name__ == "__main__":
    dirs = ["../data/txt/", "../data/img/"]
    for dir in dirs:
        print(f"Processing folder {dir}...")
        process_folder(dir)
    plt.show()
