import json
import base64


def main():
    data = ""

    with open("./out.json", "r", encoding="utf-8") as file:
        data = json.loads(file.read())

    fdata = data["file"]
    ext = data["metadata"]["file_ext"]
    name = data["metadata"]["file_name"]
    decoded_file = base64.b64decode(str(fdata))

    print(len(decoded_file))

    with open(f"{name}.{ext}", "wb") as file:
        file.write(decoded_file)


if __name__ == "__main__":
    main()
