import random

def main():
    string = input("Input string:")
    print(" ".join(map(lambda word: "".join(random.sample(word, len(word))), string.split(" "))))

if __name__ == "__main__":
    main()