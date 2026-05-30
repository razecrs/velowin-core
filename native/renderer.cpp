#include <windows.h>
#include <dwmapi.h>
#include <dcomp.h>
#include <d3d11.h>
#include <dxgi1_2.h>
#include <vector>

#pragma comment(lib, "dcomp.lib")
#pragma comment(lib, "d3d11.lib")
#pragma comment(lib, "dxgi.lib")

// The "Wayland for Windows" Compositor State
struct CompositorState {
    ID3D11Device* d3dDevice = nullptr;
    IDXGIDevice* dxgiDevice = nullptr;
    IDCompositionDevice* dcompDevice = nullptr;
    IDCompositionTarget* dcompTarget = nullptr;
    IDCompositionVisual* rootVisual = nullptr;
};

static CompositorState g_State;

extern "C" {
    __declspec(dllexport) bool InitCompositor(HWND overlayHwnd) {
        // 1. Create DirectX Device
        D3D_FEATURE_LEVEL featureLevel;
        HRESULT hr = D3D11CreateDevice(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr, D3D11_CREATE_DEVICE_BGRA_SUPPORT, 
                                       nullptr, 0, D3D11_SDK_VERSION, &g_State.d3dDevice, &featureLevel, nullptr);
        if (FAILED(hr)) return false;

        // 2. Get DXGI Device for DirectComposition
        hr = g_State.d3dDevice->QueryInterface(__uuidof(IDXGIDevice), (void**)&g_State.dxgiDevice);
        if (FAILED(hr)) return false;

        // 3. Create DirectComposition Device (Our "Wayland" Engine)
        hr = DCompositionCreateDevice(g_State.dxgiDevice, __uuidof(IDCompositionDevice), (void**)&g_State.dcompDevice);
        if (FAILED(hr)) return false;

        // 4. Bind to our transparent overlay window
        hr = g_State.dcompDevice->CreateTargetForHwnd(overlayHwnd, true, &g_State.dcompTarget);
        if (FAILED(hr)) return false;

        // 5. Create the Root Visual (The main canvas)
        hr = g_State.dcompDevice->CreateVisual(&g_State.rootVisual);
        if (FAILED(hr)) return false;

        g_State.dcompTarget->SetRoot(g_State.rootVisual);
        g_State.dcompDevice->Commit();

        return true;
    }

    __declspec(dllexport) void CreateBorder(HWND targetHwnd) {
        // Logic to create a visual node attached to our root that acts as a border
    }

    __declspec(dllexport) void UpdateBorderPosition(HWND targetHwnd, int x, int y, int width, int height) {
        // Logic to animate the visual node via the GPU
    }
}
