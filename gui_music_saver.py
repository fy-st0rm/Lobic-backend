import tkinter as tk
from tkinter import filedialog
import json
from urllib import request
from urllib.error import URLError
from tkinter import messagebox
import socket

# Getting server ip
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.connect(("8.8.8.8", 80))
server_ip = s.getsockname()[0]


#you might think that this is obselete or we dont need bloated python to corrupt our state of the art rust eco system,  but i dont care what you think!

def select_folder():
    folder_path = filedialog.askdirectory()
    if folder_path:
        path_label.config(text=f"Selected: {folder_path}")
        submit_button.config(state='normal')
        submit_button.config(command=lambda: submit_folder(folder_path))

def submit_folder(folder_path):
    try:
        data = json.dumps({'path': folder_path}).encode('utf-8')
        req = request.Request(
            f'http://{server_ip}:8080/save_music',
            data=data,
            headers={'Content-Type': 'application/json'}
        )
        
        with request.urlopen(req) as response:
            if response.status == 200 or response.status == 206:
                messagebox.showinfo("Success", "Folder path submitted successfully!")
            else:
                messagebox.showerror("Error", f"Server returned status: {response.status}")
    
    except URLError:
        messagebox.showerror("Error", "Could not connect to server. Is it running?")
    except Exception as e:
        messagebox.showerror("Error", f"An error occurred: {str(e)}")

# Create main window
root = tk.Tk()
root.title("Folder Selector")
root.minsize(400, 150)

# Create and pack widgets with padding
frame = tk.Frame(root, padx=20, pady=20)
frame.pack(expand=True, fill='both')

path_label = tk.Label(frame, text="No folder selected", wraplength=350)
path_label.pack(pady=(0, 10))

select_button = tk.Button(frame, text="Select Folder", command=select_folder)
select_button.pack(pady=(0, 10))

submit_button = tk.Button(frame, text="Submit", state='disabled')
submit_button.pack()

# Start the application
root.mainloop()
