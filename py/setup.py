import os

if __name__ == "__main__":
    path_to_venv = os.path.dirname(__file__) + "/.venv"
    path_to_requirements = os.path.dirname(__file__) + "/requirements.txt"
    if not os.path.isdir(path_to_venv):
        print("Virtual environment not detected, proceeding to create one...")
        os.system("python -m venv " + path_to_venv)
        print("Virtual environment setup successful!")

    print("Installing dependencies...")
    if os.system(path_to_venv + "/Scripts/activate && pip install -r " + path_to_requirements) == 0:
        print("Installing dependencies was successful!")
