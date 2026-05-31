import os
from PIL import Image
def convert_specific_ppm():
    input_filename = "output.ppm"
    output_dir = "renders"
    output_filename = "output.png"
    current_dir = os.path.dirname(os.path.abspath(__file__)) if __file__ in locals() else os.getcwd()
    input_path = os.path.join(current_dir, input_filename)
    output_path = os.path.join(current_dir, output_dir, output_filename)
    if not os.path.exists(input_path):
        print(f"Error: The file '{input_filename}' was not found in this folder. Stopping script.")
        return
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    try:
        with Image.open(input_path) as img:
            img.save(output_path, 'PNG')
            print(f"Success! Converted '{input_filename}' and saved to '{output_dir}/{output_filename}'")
    except Exception as e:
        print(f"An error occurred during conversion: {e}")
if __name__ == "__main__":
    convert_specific_ppm()
