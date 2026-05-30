#include <windows.h>
#include <dwmapi.h>
#include <dcomp.h>
#include <d3d11.h>
#include <dxgi1_2.h>
#include <d2d1_3.h>
#include <vector>
#include <unordered_map>

#pragma comment(lib, "dcomp.lib")
#pragma comment(lib, "d3d11.lib")
#pragma comment(lib, "dxgi.lib")
#pragma comment(lib, "d2d1.lib")

struct BorderData {
    IDCompositionVisual* visual;
    ID2D1Bitmap1* bitmap;
    HWND targetHwnd;
    int borderSize;
    float rounding;
};

// The "Wayland for Windows" Compositor State
struct CompositorState {
    ID3D11Device* d3dDevice = nullptr;
    ID3D11DeviceContext* d3dContext = nullptr;
    IDXGIDevice* dxgiDevice = nullptr;
    ID2D1Factory3* d2dFactory = nullptr;
    ID2D1Device2* d2dDevice = nullptr;
    ID2D1DeviceContext2* d2dContext = nullptr;
    
    IDCompositionDevice* dcompDevice = nullptr;
    IDCompositionTarget* dcompTarget = nullptr;
    IDCompositionVisual* rootVisual = nullptr;

    std::unordered_map<HWND, BorderData> activeBorders;
};

static CompositorState g_State;

extern "C" {
    __declspec(dllexport) bool InitCompositor(HWND overlayHwnd) {
        // 1. Create DirectX 11 Device
        D3D_FEATURE_LEVEL featureLevel;
        HRESULT hr = D3D11CreateDevice(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr, D3D11_CREATE_DEVICE_BGRA_SUPPORT, 
                                       nullptr, 0, D3D11_SDK_VERSION, &g_State.d3dDevice, &featureLevel, &g_State.d3dContext);
        if (FAILED(hr)) return false;

        hr = g_State.d3dDevice->QueryInterface(__uuidof(IDXGIDevice), (void**)&g_State.dxgiDevice);
        if (FAILED(hr)) return false;

        // 2. Create Direct2D Factory & Device (For drawing the actual rounded borders)
        D2D1_FACTORY_OPTIONS options = {};
        hr = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, __uuidof(ID2D1Factory3), &options, (void**)&g_State.d2dFactory);
        if (FAILED(hr)) return false;

        hr = g_State.d2dFactory->CreateDevice(g_State.dxgiDevice, &g_State.d2dDevice);
        if (FAILED(hr)) return false;

        hr = g_State.d2dDevice->CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE, &g_State.d2dContext);
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

    __declspec(dllexport) void CreateBorder(HWND targetHwnd, int borderSize, float rounding) {
        if (g_State.activeBorders.count(targetHwnd)) return;

        BorderData data = {};
        data.targetHwnd = targetHwnd;
        data.borderSize = borderSize;
        data.rounding = rounding;

        HRESULT hr = g_State.dcompDevice->CreateVisual(&data.visual);
        if (SUCCEEDED(hr)) {
            g_State.rootVisual->AddVisual(data.visual, TRUE, nullptr);
            g_State.activeBorders[targetHwnd] = data;
        }
    }

    __declspec(dllexport) void UpdateBorderPosition(HWND targetHwnd, int x, int y, int width, int height) {
        auto it = g_State.activeBorders.find(targetHwnd);
        if (it == g_State.activeBorders.end()) return;

        BorderData& data = it->second;
        
        // Offset the border visual to match the window position, accounting for border thickness
        data.visual->SetOffsetX((float)(x - data.borderSize));
        data.visual->SetOffsetY((float)(y - data.borderSize));
        
        // --- Direct2D Drawing Logic ---
        // Create a surface for the visual if it doesn't exist
        IDCompositionSurface* surface = nullptr;
        HRESULT hr = g_State.dcompDevice->CreateSurface(width + (data.borderSize * 2), height + (data.borderSize * 2), 
                                                        DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_ALPHA_MODE_PREMULTIPLIED, &surface);
        
        if (SUCCEEDED(hr)) {
            POINT offset;
            ID2D1DeviceContext* dc = nullptr;
            hr = surface->BeginDraw(nullptr, __uuidof(ID2D1DeviceContext), (void**)&dc, &offset);
            
            if (SUCCEEDED(hr)) {
                dc->Clear(D2D1::ColorF(0, 0, 0, 0));
                
                ID2D1SolidColorBrush* brush = nullptr;
                dc->CreateSolidColorBrush(D2D1::ColorF(D2D1::ColorF::RoyalBlue), &brush);
                
                D2D1_ROUNDED_RECT roundedRect = D2D1::RoundedRect(
                    D2D1::RectF((float)data.borderSize, (float)data.borderSize, (float)(width + data.borderSize), (float)(height + data.borderSize)),
                    data.rounding, data.rounding
                );
                
                dc->DrawRoundedRectangle(roundedRect, brush, (float)data.borderSize);
                
                if (brush) brush->Release();
                surface->EndDraw();
                dc->Release();
            }
            
            data.visual->SetContent(surface);
            surface->Release();
        }

        g_State.dcompDevice->Commit();
    }
}
