import { theme } from 'antd';

const { useToken } = theme;

export const useAntToken = () => {
  const { token } = useToken();
  return token;
};
