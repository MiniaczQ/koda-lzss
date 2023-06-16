import re
import pandas as pd
import numpy as np 
import seaborn as sns
import matplotlib.pyplot as plt

def read_file(file_name: str) -> list[str]:
    with open(file_name, "r") as results:
        results = results.readlines()
    return results

def parse_sentences(sentences):
    file_pattern = r'Testing file ./data(.+?)\.\.\.'
    compressed_pattern = r'Compressed (\d+) bytes into (\d+) bytes.'
    real_time_pattern = r'real\t(.+)'
    user_time_pattern = r'user\t(.+)'
    sys_time_pattern = r'sys\t(.+)'
    run_pattern = r'\[Running tests for code words with (\d+)-bit distance and (\d+)-bit length\]'
    coding_check = "(coding...)"
    decoding_check = "(decoding...)"
    
    is_coding = True
    current_result = None
    results = []

    distance = None
    length = None

    for sentence in sentences:
        run_match = re.search(run_pattern, sentence)
        file_match = re.search(file_pattern, sentence)
        compressed_match = re.search(compressed_pattern, sentence)
        rtime_match = re.search(real_time_pattern, sentence)
        utime_match = re.search(user_time_pattern, sentence)
        stime_match = re.search(sys_time_pattern, sentence)
        coding_match = re.search(coding_check, sentence)
        decoding_match = re.search(decoding_check, sentence)

        if run_match:
            distance = int(run_match.group(1))
            length = int(run_match.group(2))
        elif file_match:
            if current_result:
                results.append(current_result)
            current_result = {'filename': file_match.group(1)}
            current_result["length"] = length
            current_result["distance"] = distance
        elif decoding_match and current_result:
            is_coding = False
        elif coding_match and current_result:
            is_coding = True
        elif compressed_match and current_result:
            current_result['original_size'] = int(compressed_match.group(1))
            current_result['compressed_size'] = int(compressed_match.group(2))
            
        elif rtime_match and current_result:
            if is_coding == True:
                current_result['real_coding_time'] = rtime_match.group(1)
            else:
                current_result['real_decoding_time'] = rtime_match.group(1)
        elif utime_match and current_result:
            if is_coding == True:
                current_result['user_coding_time'] = utime_match.group(1)
            else:
                current_result['user_decoding_time'] = utime_match.group(1)
        elif stime_match and current_result:
            if is_coding == True:
                current_result['system_coding_time'] = stime_match.group(1)
            else:
                current_result['system_decoding_time'] = stime_match.group(1)
            
    if current_result:
        results.append(current_result)

    return results

def draw_heatmap_for_file(file_name:str, df:pd.DataFrame) -> None:
    sub_df = df[df.filename == file_name]
    table = sub_df.pivot(index="length", columns="distance", values="compression_rate")
    
    ax = sns.heatmap(table, vmax=3.5, vmin=0, cmap='RdYlGn_r', linewidths=0.5, annot=True)
    ax.invert_yaxis()
    plt.title("Compression Rate for: " + file_name)
    plt.xlabel('offset size [b]')
    plt.ylabel('match size [b]')
    plt.show()
    
def cr_heatmap_generation( df:pd.DataFrame) -> None:
    df["compression_rate"] = round(df["original_size"]/df["compressed_size"],3)
    files_list = df.filename.unique()
    for file in files_list:
        draw_heatmap_for_file(file, df)


if __name__== "__main__":
    file_name = "results_4_to_12_bits.txt"
    
    lines = read_file(file_name)
    results = parse_sentences(lines)
      
    df = pd.DataFrame.from_records(results)
    
    # CR heatmap generation
    cr_heatmap_generation(df)
    
    
    
    
    
    