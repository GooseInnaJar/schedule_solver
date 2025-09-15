import requests
import argparse
import sys
import json

def main():
    parser = argparse.ArgumentParser()

    parser.add_argument(
        "json_file", 
        help="Path to the JSON input file (e.g., ../examples/test_input_1.json)"
    )
    args = parser.parse_args()

    url = "http://localhost:8080/v1/schedule/solve"
    headers = {
        "Content-Type": "application/json"
    }

    try:
        with open(args.json_file, 'r') as f:
            json_data_string = f.read()
        json.loads(json_data_string)
    except FileNotFoundError:
        print(f"the file '{args.json_file}' was not found.", file=sys.stderr)
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"the file '{args.json_file}' contains invalid JSON. Details: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"an error occurred while reading the file: {e}", file=sys.stderr)
        sys.exit(1)

    print(f"Sending request to {url} with data from '{args.json_file}'...")
    try:
        response = requests.post(url, headers=headers, data=json_data_string)
        
        #print status
        response.raise_for_status()

        print(json.dumps(response.json(), indent=2))

    except requests.exceptions.ConnectionError:
        print(f"connection to the server at {url} failed.", file=sys.stderr)
        sys.exit(1)
    except requests.exceptions.RequestException as e:
        print("error occurred during the request: {e}", file=sys.stderr)
        if e.response is not None:
            print(f"    Response Body: {e.response.text}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()