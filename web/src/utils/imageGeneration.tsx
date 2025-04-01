import { BusterMessageData, IBusterThreadMessage } from '@/context/Threads';
import { downloadImageData, exportElementToImage } from './exportUtils';
import { createRoot } from 'react-dom/client';
import { timeout } from './timeout';
import { BusterLogoWithTextBW } from '@/assets/svg/BusterLogoWithTextBW';
import React, { useEffect, useRef } from 'react';
import { BusterChart } from '@/components/charts';

export const generateChartDownloadImage = async (
  message: IBusterThreadMessage,
  messageData: NonNullable<BusterMessageData['data']>,
  isDark: boolean = false
) => {
  const tempContainer = document.createElement('div');
  tempContainer.style.position = 'absolute';
  tempContainer.style.top = '0px';
  tempContainer.style.left = '0px';
  tempContainer.style.zIndex = '-1'; //-1
  tempContainer.style.left = '0px';
  document.body.appendChild(tempContainer);

  let container: React.ReactNode;
  let root: any;

  const darkClass = isDark
    ? 'from-slate-950 via-slate-800 to-slate-900'
    : 'from-stone-50 via-stone-100 to-stone-200';

  const renderChart: Promise<void> = new Promise((resolve) => {
    container = (
      // bg-gradient-to-br ${darkClass}
      <div className={`h-[655px] w-[880px]`}>
        <div className="relative flex h-full w-full items-center justify-center p-0">
          <BusterChart
            bordered={false}
            data={messageData}
            animate={false}
            onChartMounted={async (chart) => {
              await timeout(10); // wait for chart to mount fully?
              chart?.resize();
              await timeout(10);
              resolve();
            }}
            columnMetadata={message?.data_metadata?.column_metadata}
            {...message?.chart_config}
          />
        </div>
      </div>
    );

    root = createRoot(tempContainer);
    root.render(container);
  });

  await renderChart;

  const dataImage = await exportElementToImage(tempContainer);
  const title = message?.title || 'Buster Chart';

  await downloadImageData(dataImage, `${title}.png`);
  await timeout(10);
  root.unmount();
  tempContainer.remove();
  return dataImage;
};

export const generateChartPreviewImage = async (
  message: {
    chart_config: IBusterThreadMessage['chart_config'];
  } | null,
  messageData: BusterMessageData,
  isDark: boolean = true
) => {
  // Create temporary container and append to body
  const tempContainer = document.createElement('div');
  tempContainer.style.position = 'absolute';
  tempContainer.style.top = '0px';
  tempContainer.style.left = '0px';
  tempContainer.style.zIndex = '-1';
  tempContainer.style.left = '0px';
  document.body.appendChild(tempContainer);

  const container = (
    <PreviewImageReactComponent
      message={message as IBusterThreadMessage}
      messageData={messageData}
      isDark={isDark}
    />
  );

  // Render container into temp div
  const root = createRoot(tempContainer);
  root.render(container);

  // Wait for render to complete
  await timeout(200);
  // Export as image
  const dataImage = await exportElementToImage(tempContainer);
  await timeout(10);

  // Cleanup
  root.unmount();
  tempContainer.remove();

  return dataImage;
};

interface DotPatternProps {
  children: React.ReactNode;
  isDark: boolean;
}

const DotPattern = ({ children, isDark }: DotPatternProps) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const container = containerRef.current;
    const canvas = canvasRef.current;
    if (!container || !canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Match container size
    canvas.width = container.offsetWidth;
    canvas.height = container.offsetHeight;

    // Draw dots
    const dotColor = isDark ? 'rgba(255,255,255,0.07)' : 'rgba(0,0,0,0.073)';
    const dotSize = 1;
    const spacing = 10;

    ctx.fillStyle = dotColor;

    for (let x = 0; x < canvas.width; x += spacing) {
      for (let y = 0; y < canvas.height; y += spacing) {
        ctx.beginPath();
        ctx.arc(x, y, dotSize, 0, Math.PI * 2);
        ctx.fill();
      }
    }
  }, [isDark]);

  return (
    <div ref={containerRef} className="relative flex h-full w-full items-center justify-center">
      <canvas ref={canvasRef} className="absolute inset-0" style={{ pointerEvents: 'none' }} />
      {children}
    </div>
  );
};

const ChartPreviewImage = ({
  message,
  messageData
}: {
  message: IBusterThreadMessage;
  messageData: BusterMessageData;
}) => {
  const data = messageData?.data || [];
  const selectedChartView = message?.chart_config?.selectedView;
  const chartOptions = message?.chart_config;

  const chart = (
    <div className="h-[450px] w-[600px]">
      {/* <BusterChartJS
        animate={false}
        data={data}
        columnLabelFormats={message.chart_config.columnLabelFormats}
        bordered={false}
        view={selectedChartView}
        colors={globalChartColors}
        {...(chartOptions as any)}
      /> */}
    </div>
  );

  return chart;
};

export const PreviewImageReactComponent: React.FC<{
  message: {
    chart_config: IBusterThreadMessage['chart_config'];
  } | null;
  messageData: BusterMessageData;
  isDark: boolean;
}> = ({ isDark = false, message, messageData }) => {
  const data = messageData?.data || [];

  if (!message?.chart_config?.selectedView) {
    return <div></div>;
  }

  const BusterLogo = (
    <div
      className="w-[165px] rounded p-1"
      style={{
        color: isDark ? 'white' : 'black',
        filter: isDark
          ? 'drop-shadow(0 0 8px rgba(255, 255, 255, 0.15))'
          : 'drop-shadow(0 0 8px rgba(0, 0, 0, 0.1))'
      }}>
      <BusterLogoWithTextBW />
    </div>
  );

  const darkGradient = 'from-slate-950 via-slate-800 to-slate-900';
  const lightGradient = 'from-stone-50 via-stone-100 to-stone-200';
  const hasData = data && data.length > 0;

  return (
    <div
      className={`relative flex h-[630px] w-[1200px] items-center justify-center bg-gradient-to-br ${
        isDark ? darkGradient : lightGradient
      } p-6`}>
      <DotPattern isDark={isDark}>
        <div
          className={`flex flex-col items-center justify-center ${
            hasData ? 'space-y-3.5' : 'space-y-1.5'
          }`}>
          {BusterLogo}
          {hasData && message?.chart_config ? (
            <ChartPreviewImage
              message={message as IBusterThreadMessage}
              messageData={messageData}
            />
          ) : (
            <div
              className={`${isDark ? 'text-stone-400' : 'text-stone-700'}`}
              style={{
                fontSize: 16
              }}>
              Open source, AI-powered analytics
            </div>
          )}
        </div>
      </DotPattern>
    </div>
  );
};
