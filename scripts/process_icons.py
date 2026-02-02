from PIL import Image
import os

source_path = "assets/textures/icon.png"
output_dir = "assets/store"
sizes = [(300, 300), (150, 150), (71, 71)]

if not os.path.exists(output_dir):
    os.makedirs(output_dir)

try:
    img = Image.open(source_path)
    print(f"Loaded image: {img.format}, {img.size}, {img.mode}")
    
    # Ensure we are in RGBA
    img = img.convert("RGBA")

    for w, h in sizes:
        # High-quality resize
        resized = img.resize((w, h), Image.Resampling.LANCZOS)
        
        # 1. Save transparent version
        filename = f"Square{w}x{h}Logo.png"
        out_path = os.path.join(output_dir, filename)
        resized.save(out_path, "PNG")
        print(f"Saved transparent: {out_path}")

        # 2. Save black background version (flattened)
        # Create a black background image
        bg = Image.new("RGBA", (w, h), (0, 0, 0, 255))
        # Composite the resized image over the black background
        combined = Image.alpha_composite(bg, resized)
        # Convert to RGB (remove alpha)
        final_opaque = combined.convert("RGB")
        
        filename_opaque = f"Square{w}x{h}Logo_BlackBG.png"
        out_path_opaque = os.path.join(output_dir, filename_opaque)
        final_opaque.save(out_path_opaque, "PNG")
        print(f"Saved opaque: {out_path_opaque}")

except Exception as e:
    print(f"Error: {e}")
