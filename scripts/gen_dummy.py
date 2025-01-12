def main():
    size = 1024 * 1024 * 1024 * 1  # 1 gb

    with open("dummy.txt", "wb") as f:
        f.write(b"0" * size)


if __name__ == "__main__":
    main()
