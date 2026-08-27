<#
.SYNOPSIS
    Graba el area de juego de la ventana ("Raycaster") y genera un GIF animado
    en screenshots/demo.gif.

.DESCRIPTION
    Usa ffmpeg (captura de escritorio con gdigrab, recortada exactamente al
    area cliente de la ventana del juego -sin barra de titulo ni bordes-, en
    dos pasadas: genera una paleta de colores optima y despues codifica el
    GIF con ella). Calcula esa region con GetClientRect/ClientToScreen (el
    mismo metodo que ya usa el juego para el mouse), asi evita el borde
    negro que deja gdigrab al capturar por titulo de ventana en pantallas
    con escalado (DPI).

.PARAMETER Seconds
    Cuantos segundos grabar. Por defecto 8.

.PARAMETER OutFile
    Donde guardar el GIF resultante. Por defecto screenshots/demo.gif (relativo
    a la raiz del proyecto).

.EXAMPLE
    1) Corre el juego en otra terminal:  cargo run --release
    2) Con el juego ya abierto, Alt+Tab a esta terminal y corre:
       powershell -ExecutionPolicy Bypass -File scripts\record_gif.ps1 -Seconds 10
    3) Alt+Tab de vuelta al juego para jugar mientras graba.
#>
param(
    [int]$Seconds = 8,
    [string]$OutFile = ""
)

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($OutFile)) {
    $OutFile = Join-Path $projectRoot "screenshots\demo.gif"
}

$ffmpeg = Get-Command ffmpeg -ErrorAction SilentlyContinue
if ($null -eq $ffmpeg) {
    $fallback = Get-ChildItem "$env:LOCALAPPDATA\Microsoft\WinGet\Packages" -Recurse -Filter "ffmpeg.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($null -eq $fallback) {
        Write-Error "No se encontro ffmpeg. Instalalo con: winget install --id Gyan.FFmpeg -e"
        exit 1
    }
    $ffmpegPath = $fallback.FullName
} else {
    $ffmpegPath = $ffmpeg.Source
}

if (-not ([System.Management.Automation.PSTypeName]"RaycasterCapture").Type) {
    Add-Type @"
using System;
using System.Runtime.InteropServices;
public class RaycasterCapture {
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
    [StructLayout(LayoutKind.Sequential)]
    public struct POINT { public int X; public int Y; }
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hWnd, out RECT lpRect);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr hWnd, ref POINT lpPoint);
}
"@
}

# Sin esto, en pantallas con escalado (DPI) las coordenadas que devuelve
# Windows para la ventana del juego no coinciden con los pixeles reales que
# hay que capturar, y el GIF sale con un borde negro alrededor.
[RaycasterCapture]::SetProcessDPIAware() | Out-Null

$windowTitle = "Raycaster"

# Se busca por proceso (Get-Process) en vez de FindWindow: es mas confiable
# cuando hay varias ventanas/terminales de otros programas corriendo (VS
# Code, etc.) que podrian interferir con la busqueda por titulo exacto.
$hwnd = [IntPtr]::Zero
for ($attempt = 0; $attempt -lt 5 -and $hwnd -eq [IntPtr]::Zero; $attempt++) {
    $proc = Get-Process | Where-Object { $_.MainWindowTitle -eq $windowTitle } | Select-Object -First 1
    if ($null -ne $proc -and $proc.MainWindowHandle -ne [IntPtr]::Zero) {
        $hwnd = $proc.MainWindowHandle
    } else {
        Start-Sleep -Milliseconds 500
    }
}
if ($hwnd -eq [IntPtr]::Zero) {
    Write-Error "No encuentro una ventana abierta con titulo '$windowTitle'. Abrila con 'cargo run' antes de grabar."
    exit 1
}

$rect = New-Object RaycasterCapture+RECT
[RaycasterCapture]::GetClientRect($hwnd, [ref]$rect) | Out-Null
$topLeft = New-Object RaycasterCapture+POINT
$topLeft.X = $rect.Left
$topLeft.Y = $rect.Top
[RaycasterCapture]::ClientToScreen($hwnd, [ref]$topLeft) | Out-Null

$offsetX = $topLeft.X
$offsetY = $topLeft.Y
$width = $rect.Right - $rect.Left
$height = $rect.Bottom - $rect.Top

if ($width -le 0 -or $height -le 0) {
    Write-Error "No pude leer el tamano de la ventana del juego (esta minimizada?)."
    exit 1
}

$rawFile = Join-Path $env:TEMP "raycaster_capture.mp4"
$paletteFile = Join-Path $env:TEMP "raycaster_palette.png"
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $OutFile) | Out-Null

Write-Host "Grabando el area de juego (${width}x${height} en $offsetX,$offsetY) por $Seconds segundos..."
& $ffmpegPath -y -f gdigrab -framerate 20 -offset_x $offsetX -offset_y $offsetY -video_size "${width}x${height}" -i desktop -t $Seconds $rawFile

Write-Host "Generando paleta de colores..."
& $ffmpegPath -y -i $rawFile -vf "fps=15,scale=480:-1:flags=lanczos,palettegen" $paletteFile

Write-Host "Codificando el GIF..."
& $ffmpegPath -y -i $rawFile -i $paletteFile -filter_complex "fps=15,scale=480:-1:flags=lanczos[x];[x][1:v]paletteuse" $OutFile

Remove-Item $rawFile, $paletteFile -ErrorAction SilentlyContinue

Write-Host "Listo: $OutFile"
