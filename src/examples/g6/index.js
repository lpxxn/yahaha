
// import G6 from '@antv/g6';

const data = {
  nodes: [
    {
      id: 'node1',
      data: {
        label: 'Node 1',
        isCore: true,
        cluster: 'c1',
        x: 150,
        y: 150,
      }
    },
    {
      id: 'node2',
      data: {
        label: 'Node 2',
        cluster: 'c2',
        x: 400,
        y: 150,
      }
    },
  ],
  edges: [
    {
      id: 'edge1',
      source: 'node1',
      target: 'node2',
      data: {
        label: 'Edge 1',
      }
    },
  ],
};

const graph = new G6.Graph({
  container: 'container',
  width: 1000,
  height: 1000,
  renderer: 'canvas',
  data,
  modes: {
    default: ['drag-node', 'drag-canvas', 'click-select', 'hover-activate', 'brush-select']
  },
  theme: {
    type: 'spec',
    base: 'light',
    specification: {
      node: {
        dataTypeField: 'cluster',
      },
    },
  },
  node: innerModel => ({
    ...innerModel,
    type: 'circle',
    data: {
      ...innerModel.data,
      labelShape: {
        text: innerModel.data.label
      },
      labelBackgroundShape: {},
      iconShape: {
        img: 'https://gw.alipayobjects.com/zos/basement_prod/012bcf4f-423b-4922-8c24-32a89f8c41ce.svg',
      },
      badgeShapes: {
        0: innerModel.data.isCore ? {
          text: '核心',
          position: 'right',
          color: '#389e0d',
        } : {
          text: '紧急',
          position: 'right',
          color: '#d4380d',
        }
      },
      animates: {
        update: [
          {
            fields: ['opacity'],
            shapeId: 'haloShape',
          },
          {
            fields: ['lineWidth'],
            shapeId: 'keyShape'
          }
        ]
      }
    }
  }),
  edge: innerModel => ({
    ...innerModel,
    type: 'line',
    data: {
      labelShape: {
        text: innerModel.data.label
      },
      labelBackgroundShape: {},
      animates: {
        update: [
          {
            fields: ['opacity'],
            shapeId: 'haloShape',
          },
          {
            fields: ['lineWidth'],
            shapeId: 'keyShape'
          }
        ]
      }
    }
  }),
});