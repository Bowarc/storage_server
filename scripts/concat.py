import os


def concatenate_files_in_directory(directory_path, output_file_path):
    """
    Concatenate all files in a directory and its subdirectories into a single output file.

    :param directory_path: The path of the directory to search for files.
    :param output_file_path: The path of the file to write the combined content to.
    """
    with open(output_file_path, 'w', encoding='utf-8') as output_file:
        # Walk through the directory tree
        for root, _, files in os.walk(directory_path):
            for file in files:
                file_path = os.path.join(root, file)
                try:
                    with open(file_path, 'r', encoding='utf-8') as input_file:
                        content = input_file.read()
                        output_file.write(f'Content from file: {file_path}\n')
                        output_file.write(content.replace(
                            " ", "").replace("\n", ""))
                        # Separator between files
                        output_file.write('\n' + '-'*80 + '\n')
                except Exception as e:
                    print(f"Could not read file {file_path}: {e}")


if __name__ == '__main__':
    # Define the directory to scan and the output file path
    directory_to_scan = './back'  # Change to your directory path
    # Change to your desired output file path
    output_file = './back.txt'

    # Run the function to concatenate files
    concatenate_files_in_directory(directory_to_scan, output_file)
    print(f"All file contents have been written to {output_file}.")
