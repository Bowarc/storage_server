import requests
import os
import sys

def upload(file_name: str):
    samples_dir = os.path.join(os.path.dirname(__file__), 'samples')

    path = os.path.join(samples_dir, file_name)

    print(f"Uploading .{f'{path}'.replace(os.getcwd(), "")}")
    
    with open(path, "r") as file:
        response = requests.put(f"http://127.0.0.1:42070/{file_name}", data = file) # Assuming a server is running at that address
        print(f"Upload status code: {response.status_code}")
        print(f"Upload response text: {response.text}")

        if response.status_code == 201: # Created
            return response.text, None
        else:
            return (), "Upload failed"
        
def download(uuid: str, file_name: str):
    out_dir = os.path.join(os.path.dirname(__file__), 'out')

    print(f"Downloading {uuid}")

    response = requests.get(f"http://127.0.0.1:42070/{uuid}") # Assuming a server is running at that address

    print(f"Download status code: {response.status_code}")

    if response.status_code !=200:
        return

    path = os.path.join(out_dir, file_name) 

    print(f"File received\nWriting to .{f'{path}'.replace(os.getcwd(), "")}")

    with open(path, "wb") as file:
        for chunk in response.iter_content(chunk_size=8192):  # 8 KB chunks
            file.write(chunk)

    print("Done\n")

def delete(uuid: str):
    print(f"Deleting {uuid}")

    response = requests.delete(f"http://127.0.0.1:42070/{uuid}") # Assuming a server is running at that address

    print(f"Delete status code: {response.status_code}\n")


def main():
    file_name = "100mb.data"
    if not os.path.exists(os.path.join(os.path.dirname(__file__), 'samples', file_name)):
        print("Please generate the sample file before running the script (sh ./examples/generate_sample.sh)")
        sys.exit(1)
    
    print("If you care about speed, check the server's log, python is a bit slow 😅\n")
    uuid, err = upload(file_name)

    if err != None:
        print(err)
        sys.exit(1)

    print() # New line for readability

    download(uuid, file_name)

    delete(uuid)

    examples_path = os.path.join(os.path.dirname(__file__))
    examples_path_short = f"{examples_path}".replace(os.getcwd(), "")

    print(f"You can use `diff .{examples_path_short}/samples/{file_name} .{examples_path_short}/out/{file_name}`\nIf you see no output, it means that the files are identical")


if __name__ == "__main__":
    main()
