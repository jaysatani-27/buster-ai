'use client';

import React, { useEffect, useMemo } from 'react';
import dayjs from 'dayjs';
import { useMemoizedFn, useNetwork } from 'ahooks';
import { Divider } from 'antd';
import { ReadyState, useBusterWebSocketConnectionStatus } from '@/context/BusterWebSocket';
import last from 'lodash/last';
import { useHotkeys } from 'react-hotkeys-hook';

interface NetworkDrop {
  timestamp: dayjs.Dayjs;
}

export const TempNetworkVisibility: React.FC = React.memo(() => {
  const status = useNetwork();
  const connectionStatus = useBusterWebSocketConnectionStatus();
  const [show, setShow] = React.useState(false);
  const { position, isDragging, handleMouseDown } = useDraggable({ initialX: 5, initialY: 0 });
  const [networkDrops, setNetworkDrops] = React.useState<NetworkDrop[]>([]);
  const [isMinimized, setIsMinimized] = React.useState(true);

  const [websocketStatuses, setWebsocketStatuses] = React.useState<
    {
      timestamp: dayjs.Dayjs;
      status: ReadyState;
    }[]
  >([]);

  const connectionStatusText = useMemoizedFn((status: ReadyState) => {
    if (status === ReadyState.Open) {
      return 'Open';
    }
    if (connectionStatus === ReadyState.Connecting) {
      return 'Connecting...';
    }
    if (connectionStatus === ReadyState.Closing) {
      return 'Closing...';
    }
    if (connectionStatus === ReadyState.Closed) {
      return 'Closed';
    }
    return 'Closed';
  });

  const MinimizeButton = useMemoizedFn(() => (
    <button
      className="absolute right-1 top-1 flex h-4 w-fit min-w-6 items-center justify-center rounded border border-black/25 bg-white/50 text-[10px] hover:bg-white/80"
      onClick={(e) => {
        e.stopPropagation();
        setIsMinimized(!isMinimized);
      }}>
      {isMinimized ? (
        <div className="w-12">
          {connectionStatusText(last(websocketStatuses)?.status || ReadyState.Closed)}-
          {websocketStatuses.length}
        </div>
      ) : (
        '−'
      )}
    </button>
  ));

  React.useEffect(() => {
    if (status.online !== true) {
      setNetworkDrops((prev) => [...prev, { timestamp: dayjs() }]);
    }
  }, [status.online]);

  useEffect(() => {
    setWebsocketStatuses((prev) => [...prev, { timestamp: dayjs(), status: connectionStatus }]);
  }, [connectionStatus]);

  useHotkeys('n+a+t+e', () => {
    setShow(true);
  });

  if (!show) return null;

  return (
    <div
      className={`fixed flex cursor-move rounded shadow ${
        isMinimized ? 'flex h-6 w-16' : 'w-fit flex-col space-y-3 px-4 py-3'
      }`}
      style={{
        right: position.x === 0 ? '500px' : undefined,
        bottom: position.y === 0 ? '5px' : undefined,
        left: position.x !== 0 ? position.x : undefined,
        top: position.y !== 0 ? position.y : undefined,
        userSelect: 'none',
        backgroundColor: 'rgba(255, 255, 200, 0.45)',
        backdropFilter: 'blur(1px)',
        border: '1px solid rgba(0, 0, 0, 0.25)',
        boxShadow: '0 0 10px rgba(255, 255, 255, 0.1)'
      }}
      onClick={() => isMinimized && setIsMinimized(false)}
      onMouseDown={handleMouseDown}>
      <MinimizeButton />
      {!isMinimized && (
        <>
          <div className="flex flex-col gap-y-0.5">
            <div className="text-sm font-bold">Websocket status {websocketStatuses.length}</div>
            <div className="flex max-h-48 flex-col gap-y-0.5 overflow-y-auto">
              {websocketStatuses.map((status) => (
                <div className="flex" key={status.timestamp.toISOString()}>
                  {status.timestamp.format('HH:mm:ss.SSS')} - {connectionStatusText(status.status)}
                </div>
              ))}
            </div>
          </div>

          <div>
            <div className="text-sm font-bold">Network status</div>
            <div className="grid grid-cols-2 gap-x-10 gap-y-1">
              <div>Status:</div>
              <div>{status.online ? 'Online' : 'Offline'}</div>
              <div>Download:</div>
              <div>{status.downlink}</div>
              <div>Effective Type:</div>
              <div>{status.effectiveType}</div>
              <div>RTT:</div>
              <div>{status.rtt}</div>
              <div>Since:</div>
              <div>{status.since ? dayjs(status.since).format('HH:mm:ss.SSS') : 'N/A'}</div>
            </div>
          </div>

          <Divider />

          <div className="flex flex-col gap-y-1">
            <div className="text-sm font-bold">Network Drops</div>
            <div className="max-h-48 overflow-y-auto">
              {networkDrops.map((drop) => (
                <div key={drop.timestamp.toISOString()}>
                  {drop.timestamp.format('HH:mm:ss.SSS')}
                </div>
              ))}
            </div>
          </div>
        </>
      )}
    </div>
  );
});
TempNetworkVisibility.displayName = 'TempNetworkVisibility';

const useDraggable = ({ initialX = 0, initialY = 0 }) => {
  const [position, setPosition] = React.useState({ x: initialX, y: initialY });
  const [isDragging, setIsDragging] = React.useState(false);
  const dragRef = React.useRef<{ startX: number; startY: number; elemX: number; elemY: number }>();

  const handleMouseDown = useMemoizedFn((e: React.MouseEvent) => {
    if ((e.target as HTMLElement).closest('button')) {
      return;
    }

    setIsDragging(true);
    const rect = e.currentTarget.getBoundingClientRect();
    dragRef.current = {
      startX: e.clientX,
      startY: e.clientY,
      elemX: rect.left,
      elemY: rect.top
    };
  });

  const handleMouseMove = useMemoizedFn((e: MouseEvent) => {
    if (!isDragging || !dragRef.current) return;

    const deltaX = e.clientX - dragRef.current.startX;
    const deltaY = e.clientY - dragRef.current.startY;

    setPosition({
      x: dragRef.current.elemX + deltaX,
      y: dragRef.current.elemY + deltaY
    });
  });

  const handleMouseUp = useMemoizedFn(() => {
    setIsDragging(false);
  });

  React.useEffect(() => {
    if (isDragging) {
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);
    }
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging, handleMouseMove, handleMouseUp]);

  return { position, isDragging, handleMouseDown, handleMouseMove, handleMouseUp };
};
