import os
import math
from PIL import Image, ImageDraw

def create_time_icon():
    # Render at high resolution (1024x1024) for supreme antialiased quality
    size = 1024
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    center = size / 2.0
    
    # Outer squircle / rounded rect badge
    margin = 48
    radius = 220
    rect = [margin, margin, size - margin, size - margin]
    
    # Outer squircle background: deep matte obsidian
    bg_color = (18, 18, 20, 255)
    border_color = (48, 48, 54, 255)
    
    draw.rounded_rectangle(rect, radius=radius, fill=bg_color, outline=border_color, width=8)

    # Subtle inner groove / clock rim
    dial_radius = 380
    dial_bbox = [center - dial_radius, center - dial_radius, center + dial_radius, center + dial_radius]
    draw.ellipse(dial_bbox, outline=(36, 36, 42, 255), width=6)
    
    # 12 clock hour ticks
    for hour in range(12):
        angle_rad = math.radians(hour * 30 - 90)
        is_cardinal = (hour % 3 == 0)
        tick_outer = dial_radius - 24
        tick_inner = tick_outer - (36 if is_cardinal else 20)
        tick_width = 10 if is_cardinal else 5
        tick_color = (240, 240, 245, 255) if is_cardinal else (110, 110, 120, 255)
        
        x0 = center + tick_inner * math.cos(angle_rad)
        y0 = center + tick_inner * math.sin(angle_rad)
        x1 = center + tick_outer * math.cos(angle_rad)
        y1 = center + tick_outer * math.sin(angle_rad)
        
        draw.line([(x0, y0), (x1, y1)], fill=tick_color, width=tick_width)

    # Subtle concentric soundwave / vinyl groove ring
    groove_radius = 260
    groove_bbox = [center - groove_radius, center - groove_radius, center + groove_radius, center + groove_radius]
    draw.ellipse(groove_bbox, outline=(28, 28, 34, 255), width=4)

    # Center clock / music motif:
    # Stylized Clock Hands combined with a sleek central Play Triangle
    # Clock hour hand pointing towards ~10 o'clock
    h_angle = math.radians(-135)
    h_len = 160
    hx = center + h_len * math.cos(h_angle)
    hy = center + h_len * math.sin(h_angle)
    draw.line([(center, center), (hx, hy)], fill=(200, 200, 208, 255), width=16)

    # Minute hand pointing towards 12 o'clock
    m_angle = math.radians(-90)
    m_len = 220
    mx = center + m_len * math.cos(m_angle)
    my = center + m_len * math.sin(m_angle)
    draw.line([(center, center), (mx, my)], fill=(255, 255, 255, 255), width=16)

    # Center Hub - Pure White circle with dark core
    hub_radius = 36
    draw.ellipse([center - hub_radius, center - hub_radius, center + hub_radius, center + hub_radius], fill=(255, 255, 255, 255))
    core_radius = 16
    draw.ellipse([center - core_radius, center - core_radius, center + core_radius, center + core_radius], fill=(18, 18, 20, 255))

    # Bold Play Triangle in lower-right quadrant / center accent (Music Player Identity)
    play_center_x = center + 55
    play_center_y = center + 65
    play_size = 90
    
    # Equilateral triangle pointing right
    p1 = (play_center_x - play_size * 0.5, play_center_y - play_size * 0.7)
    p2 = (play_center_x + play_size * 0.7, play_center_y)
    p3 = (play_center_x - play_size * 0.5, play_center_y + play_size * 0.7)
    
    draw.polygon([p1, p2, p3], fill=(255, 255, 255, 255))

    return img

def main():
    icons_dir = r"C:\Users\Strelok\.gemini\antigravity\scratch\Time\src-tauri\icons"
    os.makedirs(icons_dir, exist_ok=True)

    master = create_time_icon()

    # Generate 512x512 for icon.png
    icon_512 = master.resize((512, 512), Image.Resampling.LANCZOS)
    icon_png_path = os.path.join(icons_dir, "icon.png")
    icon_512.save(icon_png_path, format="PNG")
    print(f"Saved: {icon_png_path}")

    # Generate 128x128.png
    icon_128 = master.resize((128, 128), Image.Resampling.LANCZOS)
    icon_128_path = os.path.join(icons_dir, "128x128.png")
    icon_128.save(icon_128_path, format="PNG")
    print(f"Saved: {icon_128_path}")

    # Generate 128x128@2x.png (256x256)
    icon_256 = master.resize((256, 256), Image.Resampling.LANCZOS)
    icon_256_path = os.path.join(icons_dir, "128x128@2x.png")
    icon_256.save(icon_256_path, format="PNG")
    print(f"Saved: {icon_256_path}")

    # Generate 32x32.png
    icon_32 = master.resize((32, 32), Image.Resampling.LANCZOS)
    icon_32_path = os.path.join(icons_dir, "32x32.png")
    icon_32.save(icon_32_path, format="PNG")
    print(f"Saved: {icon_32_path}")

    # Generate icon.ico containing multi-resolutions: 16, 24, 32, 48, 64, 128, 256
    icon_ico_path = os.path.join(icons_dir, "icon.ico")
    master.save(
        icon_ico_path,
        format="ICO",
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    )
    print(f"Saved: {icon_ico_path}")

if __name__ == "__main__":
    main()
