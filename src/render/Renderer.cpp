#include <windows.h>
#include <dwmapi.h>
#include <dcomp.h>
#include <d3d11.h>
#include <dxgi1_2.h>
#include <d2d1_3.h>
#include <d2d1effects_2.h>
#include <vector>
#include <unordered_map>
#include <cmath>
#include <algorithm>

#pragma comment(lib, "dcomp.lib")
#pragma comment(lib, "d3d11.lib")
#pragma comment(lib, "dxgi.lib")
#pragma comment(lib, "d2d1.lib")
#pragma comment(lib, "dxguid.lib")

#define M_SQRT2 1.41421356237309504880f

struct Color {
    float r, g, b, a;
};

struct BorderData {
    IDCompositionVisual* visual = nullptr;
    IDCompositionVisual* shadowVisual = nullptr;
    HWND targetHwnd = nullptr;
    int borderSize = 2;
    float rounding = 10.0f;
    float angle = 0.0f;
    std::vector<Color> colors;
    
    bool shadowEnabled = true;
    int shadowRange = 15;
    int shadowPower = 3;
    Color shadowColor = {0.0f, 0.0f, 0.0f, 0.5f};
};

struct CompositorState {
    ID3D11Device* d3dDevice = nullptr;
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
        D3D_FEATURE_LEVEL featureLevel;
        HRESULT hr = D3D11CreateDevice(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr, D3D11_CREATE_DEVICE_BGRA_SUPPORT, 
                                       nullptr, 0, D3D11_SDK_VERSION, &g_State.d3dDevice, &featureLevel, nullptr);
        if (FAILED(hr)) return false;
        hr = g_State.d3dDevice->QueryInterface(__uuidof(IDXGIDevice), (void**)&g_State.dxgiDevice);
        if (FAILED(hr)) return false;
        D2D1_FACTORY_OPTIONS options = {};
        hr = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, __uuidof(ID2D1Factory3), &options, (void**)&g_State.d2dFactory);
        if (FAILED(hr)) return false;
        hr = g_State.d2dFactory->CreateDevice(g_State.dxgiDevice, &g_State.d2dDevice);
        if (FAILED(hr)) return false;
        hr = g_State.d2dDevice->CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE, &g_State.d2dContext);
        if (FAILED(hr)) return false;
        hr = DCompositionCreateDevice(g_State.dxgiDevice, __uuidof(IDCompositionDevice), (void**)&g_State.dcompDevice);
        if (FAILED(hr)) return false;
        hr = g_State.dcompDevice->CreateTargetForHwnd(overlayHwnd, true, &g_State.dcompTarget);
        if (FAILED(hr)) return false;
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
        data.colors = {{0.25f, 0.41f, 0.88f, 1.0f}, {0.0f, 0.0f, 0.0f, 1.0f}};
        g_State.dcompDevice->CreateVisual(&data.visual);
        g_State.dcompDevice->CreateVisual(&data.shadowVisual);
        g_State.rootVisual->AddVisual(data.shadowVisual, FALSE, nullptr);
        g_State.rootVisual->AddVisual(data.visual, TRUE, data.shadowVisual);
        g_State.activeBorders[targetHwnd] = data;
    }

    __declspec(dllexport) void SetBorderAngle(HWND targetHwnd, float angle) {
        auto it = g_State.activeBorders.find(targetHwnd);
        if (it != g_State.activeBorders.end()) it->second.angle = angle;
    }

    __declspec(dllexport) void UpdateBorderPosition(HWND targetHwnd, int x, int y, int width, int height) {
        auto it = g_State.activeBorders.find(targetHwnd);
        if (it == g_State.activeBorders.end()) return;
        BorderData& data = it->second;

        // 1:1 Hyprland rounding correction math
        const float roundingPower = 2.0f; // default
        const float correctionOffset = (data.borderSize * (M_SQRT2 - 1.0f) * (std::max(2.0f - roundingPower, 0.0f)));
        const float outerRound = (data.rounding + data.borderSize) - correctionOffset;

        float vX = (float)(x - data.borderSize);
        float vY = (float)(y - data.borderSize);
        int sW = width + (data.borderSize * 2);
        int sH = height + (data.borderSize * 2);

        data.visual->SetOffsetX(vX);
        data.visual->SetOffsetY(vY);

        IDCompositionSurface* surface = nullptr;
        if (SUCCEEDED(g_State.dcompDevice->CreateSurface(sW, sH, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_ALPHA_MODE_PREMULTIPLIED, &surface))) {
            POINT offset; ID2D1DeviceContext* dc = nullptr;
            if (SUCCEEDED(surface->BeginDraw(nullptr, __uuidof(ID2D1DeviceContext), (void**)&dc, &offset))) {
                dc->Clear(D2D1::ColorF(0, 0, 0, 0));
                std::vector<D2D1_GRADIENT_STOP> stops;
                for (size_t i = 0; i < data.colors.size(); ++i) {
                    stops.push_back({ (float)i / (data.colors.size() - 1), D2D1::ColorF(data.colors[i].r, data.colors[i].g, data.colors[i].b, data.colors[i].a) });
                }
                ID2D1GradientStopCollection* pStopCollection = nullptr;
                dc->CreateGradientStopCollection(stops.data(), (UINT32)stops.size(), &pStopCollection);
                if (pStopCollection) {
                    float rad = data.angle * (3.14159f / 180.0f);
                    float cx = sW / 2.0f; float cy = sH / 2.0f;
                    float len = sqrtf((float)sW * sW + sH * sH);
                    D2D1_POINT_2F start = D2D1::Point2F(cx - cosf(rad) * len / 2, cy - sinf(rad) * len / 2);
                    D2D1_POINT_2F end = D2D1::Point2F(cx + cosf(rad) * len / 2, cy + sinf(rad) * len / 2);
                    ID2D1LinearGradientBrush* pBrush = nullptr;
                    dc->CreateLinearGradientBrush(D2D1::LinearGradientBrushProperties(start, end), pStopCollection, &pBrush);
                    if (pBrush) {
                        D2D1_ROUNDED_RECT roundedRect = D2D1::RoundedRect(D2D1::RectF((float)data.borderSize/2, (float)data.borderSize/2, (float)sW - data.borderSize/2, (float)sH - data.borderSize/2), outerRound, outerRound);
                        dc->DrawRoundedRectangle(roundedRect, (ID2D1Brush*)pBrush, (float)data.borderSize);
                        pBrush->Release();
                    }
                    pStopCollection->Release();
                }
                surface->EndDraw(); dc->Release();
            }
            data.visual->SetContent(surface); surface->Release();
        }

        if (data.shadowEnabled) {
            int shW = sW + (data.shadowRange * 2);
            int shH = sH + (data.shadowRange * 2);
            data.shadowVisual->SetOffsetX(vX - data.shadowRange);
            data.shadowVisual->SetOffsetY(vY - data.shadowRange);
            IDCompositionSurface* shSurface = nullptr;
            if (SUCCEEDED(g_State.dcompDevice->CreateSurface(shW, shH, DXGI_FORMAT_B8G8_UNORM, DXGI_ALPHA_MODE_PREMULTIPLIED, &shSurface))) {
                // TODO: Shadow drawing logic
                shSurface->Release();
            }
        }
        g_State.dcompDevice->Commit();
    }
}
